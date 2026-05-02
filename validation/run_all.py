#!/usr/bin/env python3
"""Validate every slug in benchmark/, then print a combined summary table.

For each subdir benchmark/<slug>/ that has a work/ directory:
  1. Run validation/validate.sh with WORK=benchmark/<slug>/work.
  2. validate.sh writes benchmark/<slug>/outputs/validation.json.

After all slugs run, print one row per slug with cost/tokens/elapsed (from the
benchmark phase's result.json) joined to pass/total (from validation.json).
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BENCH = ROOT / "benchmark"
VALIDATE_SH = Path(__file__).resolve().parent / "validate.sh"


def slug_dirs() -> list[Path]:
    if not BENCH.is_dir():
        return []
    return sorted(p for p in BENCH.iterdir() if p.is_dir() and (p / "work").is_dir())


def read_json(path: Path, default=None):
    try:
        return json.loads(path.read_text())
    except (FileNotFoundError, json.JSONDecodeError):
        return default


def run_validate(slug_dir: Path) -> int:
    work = slug_dir / "work"
    print(f"\n==================== {slug_dir.name} ====================", flush=True)
    proc = subprocess.run(
        ["bash", str(VALIDATE_SH)],
        env={**os.environ, "WORK": str(work)},
    )
    return proc.returncode


def status_label(v: dict | None) -> str:
    if not v:
        return "—"
    if v.get("build_ok") is False:
        return "BUILD-FAIL"
    return f"{v.get('pass', 0)}/{v.get('total', 0)}"


def print_table(slugs: list[Path]) -> None:
    rows = []
    for slug_dir in slugs:
        outputs = slug_dir / "outputs"
        result = read_json(outputs / "result.json", default={})
        valid = read_json(outputs / "validation.json")
        rows.append({
            "slug":    slug_dir.name,
            "cost":    float(result.get("cost_usd") or 0),
            "tokens":  int((result.get("tokens") or {}).get("total") or 0),
            "elapsed": float(result.get("elapsed_seconds") or 0),
            "pass":    status_label(valid),
        })

    w = max(len("model"), max(len(r["slug"]) for r in rows))
    fmt = f"{{:<{w}}}  {{:>9}}  {{:>9}}  {{:>8}}  {{:>10}}"
    sep = "-" * (w + 46)
    print()
    print(fmt.format("model", "cost", "tokens", "elapsed", "pass"))
    print(sep)
    total_cost = 0.0
    for r in rows:
        print(fmt.format(
            r["slug"], f"${r['cost']:.4f}", f"{r['tokens']:,}",
            f"{r['elapsed']:.0f}s", r["pass"],
        ))
        total_cost += r["cost"]
    print(sep)
    print(fmt.format("TOTAL", f"${total_cost:.4f}", "", "", ""))
    print()


def main() -> int:
    slugs = slug_dirs()
    if not slugs:
        sys.exit(f"no slug subdirs under {BENCH}")
    if not VALIDATE_SH.is_file():
        sys.exit(f"missing {VALIDATE_SH}")

    failures = 0
    for slug_dir in slugs:
        if run_validate(slug_dir) != 0:
            failures += 1

    print("\n==================== VALIDATION SUMMARY ====================")
    print_table(slugs)
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
