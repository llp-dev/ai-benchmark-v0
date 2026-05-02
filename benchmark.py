#!/usr/bin/env python3
"""Run the AI benchmark.

Layout (relative to this script):
    benchmark/
        config.json                 image, timeout, slugs to run
        prompt.md                   shared spec, bind-mounted read-only at /work/prompt.txt
        <slug>/
            opencode.json           per-slug config; carries its own "model" field
            work/                   bind-mounted as /work inside the container
            outputs/                stdout.log, stderr.log, result.json
"""

from __future__ import annotations

import json
import os
import shutil
import sys
import threading
import time
import uuid
from datetime import datetime, timezone
from pathlib import Path

import docker
from docker.errors import APIError, NotFound

ROOT = Path(__file__).resolve().parent
BENCH = ROOT / "benchmark"
CONFIG = BENCH / "config.json"
PROMPT = BENCH / "prompt.md"


# ------------------------------- common helpers -------------------------------

def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def log(msg: str) -> None:
    print(f"[{datetime.now().strftime('%H:%M:%S')}] {msg}", flush=True)


def write_json(path: Path, data: dict) -> None:
    path.write_text(json.dumps(data, indent=2) + "\n")


def read_json(path: Path, default=None):
    try:
        return json.loads(path.read_text())
    except (FileNotFoundError, json.JSONDecodeError):
        return default


_docker_client = None


def docker_client():
    global _docker_client
    if _docker_client is None:
        _docker_client = docker.from_env()
    return _docker_client


def start_container(image: str, command: list[str], *,
                    volumes: dict | None = None,
                    environment: dict | None = None,
                    name: str | None = None,
                    workdir: str = "/work"):
    """Start a detached container. Caller must container.remove(force=True)."""
    return docker_client().containers.run(
        image, command,
        name=name,
        volumes=volumes or {},
        environment=environment or {},
        working_dir=workdir,
        detach=True,
    )


def remove_quiet(container) -> None:
    try:
        container.remove(force=True)
    except (NotFound, APIError):
        pass


def iter_demuxed(stream):
    """Yield ('out'|'err', line) pairs from a demuxed log stream, splitting on newlines."""
    out_buf = bytearray()
    err_buf = bytearray()

    def flush(buf: bytearray, kind: str):
        while True:
            idx = buf.find(b"\n")
            if idx < 0:
                return
            line = bytes(buf[:idx + 1]).decode("utf-8", errors="replace")
            del buf[:idx + 1]
            yield kind, line

    for stdout_chunk, stderr_chunk in stream:
        if stdout_chunk:
            out_buf.extend(stdout_chunk)
            yield from flush(out_buf, "out")
        if stderr_chunk:
            err_buf.extend(stderr_chunk)
            yield from flush(err_buf, "err")
    if out_buf:
        yield "out", bytes(out_buf).decode("utf-8", errors="replace")
    if err_buf:
        yield "err", bytes(err_buf).decode("utf-8", errors="replace")


def load_dotenv(path: Path) -> None:
    if not path.is_file():
        return
    for line in path.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        k, _, v = line.partition("=")
        os.environ.setdefault(k.strip(), v.strip().strip('"').strip("'"))


def load_config() -> dict:
    if not CONFIG.is_file():
        sys.exit(f"missing {CONFIG}")
    return json.loads(CONFIG.read_text())


# ------------------------------ benchmark phase ------------------------------

def already_done(outputs: Path) -> bool:
    d = read_json(outputs / "result.json", default={})
    return d.get("exit_code") == 0 and not d.get("timed_out")


def parse_ndjson(path: Path) -> dict:
    """Pull session_id, finish_reason, cumulative tokens, and total cost from
    opencode's NDJSON. Tokens live in step_finish events under part.tokens
    (input/output are per-step; total is cumulative). Cost is per-step in
    part.cost — sum across all steps for the run total.
    """
    sid = reason = None
    tokens = {"input": 0, "output": 0, "total": 0}
    cost = 0.0
    if not path.is_file():
        return {"session_id": None, "finish_reason": None, "tokens": tokens, "cost_usd": 0.0}
    with path.open(errors="replace") as fh:
        for raw in fh:
            if not raw.startswith("{"):
                continue
            try:
                ev = json.loads(raw)
            except json.JSONDecodeError:
                continue
            sid = ev.get("sessionID") or sid
            if ev.get("type") != "step_finish":
                continue
            part = ev.get("part") or {}
            reason = part.get("reason") or reason
            t = part.get("tokens") or {}
            tokens["input"] += int(t.get("input") or 0)
            tokens["output"] += int(t.get("output") or 0)
            if isinstance(t.get("total"), (int, float)):
                tokens["total"] = int(t["total"])
            if isinstance(part.get("cost"), (int, float)):
                cost += float(part["cost"])
    return {"session_id": sid, "finish_reason": reason, "tokens": tokens, "cost_usd": round(cost, 6)}


def _snippet(text: str, n: int = 110) -> str:
    s = text.strip().replace("\n", " ")
    return s if len(s) <= n else s[: n - 3] + "..."


def summarize_event(ev: dict) -> str | None:
    """One-line description for the live ticker. Returns None to stay silent."""
    kind = ev.get("type")
    part = ev.get("part") or {}
    if kind == "tool_use":
        state = part.get("state") or {}
        args = state.get("input") or {}
        hint = next(
            (args[k][:60] for k in ("filePath", "path", "file", "command", "cmd", "pattern")
             if isinstance(args.get(k), str) and args[k]),
            "",
        )
        bits = ["tool", part.get("tool", "?")]
        if state.get("status"):
            bits.append(state["status"])
        if hint:
            bits.append(hint)
        return " ".join(bits)
    if kind == "text":
        text = part.get("text") or ""
        return f"text:      {_snippet(text)}" if text.strip() else None
    if kind == "reasoning":
        text = part.get("text") or ""
        return f"reasoning: {_snippet(text)}" if text.strip() else None
    if kind == "step_finish":
        t = part.get("tokens") or {}
        return f"step_finish total={t.get('total', '?')} reason={part.get('reason', '?')}"
    return None if kind in ("step_start", "snapshot", "patch", "agent") else kind


# Single-line bootstrap. The full prompt lives in /work/prompt.txt; the agent's
# first move is to read it. Must be inside /work because opencode rejects
# directory access outside its cwd even with yolo on.
PROMPT_BOOTSTRAP = "Read ./prompt.txt and follow the instructions inside it exactly. ./prompt.txt is read-only spec; do not modify or delete it."


def stream(container, stdout_log: Path, stderr_log: Path,
           timeout: int, tag: str) -> tuple[int, bool]:
    """Tee container stdout/stderr to files, print live ticker + heartbeat,
    kill on timeout. Returns (exit_code, timed_out)."""
    timed_out = False
    last_event = time.monotonic()
    stop_heartbeat = threading.Event()

    def kill_on_timeout():
        nonlocal timed_out
        timed_out = True
        log(f"  {tag} | !!! TIMEOUT after {timeout}s — killing container")
        try:
            container.kill()
        except (NotFound, APIError):
            pass

    timer = threading.Timer(timeout, kill_on_timeout)
    timer.daemon = True
    timer.start()

    def heartbeat():
        while not stop_heartbeat.is_set():
            stop_heartbeat.wait(15)
            if stop_heartbeat.is_set():
                break
            silent = time.monotonic() - last_event
            if silent >= 30:
                log(f"  {tag} | … still running ({silent:.0f}s since last event)")

    threading.Thread(target=heartbeat, daemon=True).start()

    last_print = 0.0
    log_stream = container.attach(stdout=True, stderr=True, stream=True, logs=True, demux=True)
    with stdout_log.open("w") as outf, stderr_log.open("w") as errf:
        for kind, line in iter_demuxed(log_stream):
            last_event = time.monotonic()
            if kind == "err":
                errf.write(line)
                errf.flush()
                continue
            outf.write(line)
            outf.flush()
            if not line.startswith("{"):
                continue
            try:
                ev = json.loads(line)
            except json.JSONDecodeError:
                continue
            summary = summarize_event(ev)
            now = time.monotonic()
            if summary and now - last_print >= 0.25:
                log(f"  {tag} | {summary}")
                last_print = now

    timer.cancel()
    stop_heartbeat.set()
    result = container.wait()
    return int(result.get("StatusCode") or 0), timed_out


def code_files(work: Path) -> dict:
    # prompt.txt is an empty stub Docker leaves on the host as the file-mount target.
    files = sorted(
        p.name for p in work.iterdir()
        if p.is_file() and p.name != "prompt.txt"
    ) if work.is_dir() else []
    return {"file_count": len(files), "files": files}


def run_one(cfg: dict, slug: str, total: tuple[int, int]) -> dict:
    slug_dir = BENCH / slug
    work = slug_dir / "work"
    outputs = slug_dir / "outputs"
    oc_cfg = slug_dir / "opencode.json"
    cur, tot = total

    if not oc_cfg.is_file():
        sys.exit(f"missing {oc_cfg}")

    if already_done(outputs):
        log(f"[skip] ({cur}/{tot}) {slug} already complete")
        return read_json(outputs / "result.json", default={})

    if work.exists():
        shutil.rmtree(work)
    work.mkdir(parents=True)
    outputs.mkdir(parents=True, exist_ok=True)

    stdout_log, stderr_log = outputs / "stdout.log", outputs / "stderr.log"
    name = f"benchmark-{slug}-{uuid.uuid4().hex[:8]}"
    timeout = int(cfg["timeout_seconds"])
    log(f"[run ] ({cur}/{tot}) {slug} timeout={timeout}s")
    log(f"        container={name}")

    started, t0 = now_iso(), time.monotonic()
    container = start_container(
        cfg["image"],
        ["opencode", "run", "--format", "json", "--thinking", PROMPT_BOOTSTRAP],
        name=name,
        volumes={
            str(work.resolve()):   {"bind": "/work", "mode": "rw"},
            str(PROMPT.resolve()): {"bind": "/work/prompt.txt", "mode": "ro"},
            str(oc_cfg.resolve()): {"bind": "/etc/opencode.json", "mode": "ro"},
        },
        environment={
            "OPENROUTER_API_KEY": os.environ["OPENROUTER_API_KEY"],
            "OPENCODE_CONFIG":    "/etc/opencode.json",
        },
    )
    try:
        exit_code, timed = stream(container, stdout_log, stderr_log, timeout, slug)
    finally:
        remove_quiet(container)
    elapsed = time.monotonic() - t0

    parsed = parse_ndjson(stdout_log)
    result = {
        "slug": slug,
        "started_at": started,
        "ended_at": now_iso(),
        "elapsed_seconds": round(elapsed, 2),
        "exit_code": exit_code,
        "timed_out": timed,
        **parsed,
        "code": code_files(work),
    }
    write_json(outputs / "result.json", result)
    if timed:
        status = "TIMEOUT"
    elif exit_code == 0:
        status = "OK"
    else:
        status = f"FAIL({exit_code})"
    log(
        f"[done] {slug} {status} elapsed={elapsed:.0f}s "
        f"files={result['code']['file_count']} tokens={parsed['tokens']['total']} "
        f"cost=${parsed['cost_usd']:.4f}"
    )
    return result


# ---------------------------------- results ----------------------------------

def print_table(slugs: list[str]) -> None:
    rows = []
    for slug in slugs:
        r = read_json(BENCH / slug / "outputs" / "result.json", default={})
        rows.append({
            "slug": slug,
            "cost": float(r.get("cost_usd") or 0),
            "tokens": int((r.get("tokens") or {}).get("total") or 0),
            "elapsed": float(r.get("elapsed_seconds") or 0),
            "finish": r.get("finish_reason") or ("crash" if not r else "?"),
        })
    if not rows:
        print("(no rows)")
        return

    rows.sort(key=lambda r: r["slug"])
    w = max(len("model"), max(len(r["slug"]) for r in rows))
    fmt = f"{{:<{w}}}  {{:>9}}  {{:>8}}  {{:>8}}  {{:<10}}"
    sep = "-" * (w + 42)
    print()
    print(fmt.format("model", "cost", "tokens", "elapsed", "finish"))
    print(sep)
    total = 0.0
    for r in rows:
        print(fmt.format(
            r["slug"], f"${r['cost']:.4f}", f"{r['tokens']:,}",
            f"{r['elapsed']:.0f}s", r["finish"],
        ))
        total += r["cost"]
    print(sep)
    print(fmt.format("TOTAL", f"${total:.4f}", "", "", ""))
    print()


# --------------------------------- entrypoint ---------------------------------

def main() -> int:
    load_dotenv(ROOT / ".env")
    if not os.environ.get("OPENROUTER_API_KEY"):
        sys.exit("OPENROUTER_API_KEY not set (export it or put it in .env)")
    if not PROMPT.is_file():
        sys.exit(f"missing {PROMPT}")
    cfg = load_config()
    slugs = cfg["slugs"]

    runs: list[dict] = []
    log("==================== BENCHMARK PHASE ====================")
    log(f"slugs={len(slugs)} timeout={cfg['timeout_seconds']}s image={cfg['image']}")
    for i, slug in enumerate(slugs, start=1):
        runs.append(run_one(cfg, slug, (i, len(slugs))))

    write_json(BENCH / "summary.json", {"runs": runs, "generated_at": now_iso()})
    log(f"wrote {BENCH.name}/summary.json ({len(runs)} runs)")
    log("==================== RESULTS ====================")
    print_table(slugs)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
