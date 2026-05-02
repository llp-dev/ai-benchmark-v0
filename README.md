# ai-benchmark-v0

Single-task LLM coding benchmark: each candidate model runs **opencode** inside an Alpine Docker container and is asked to implement the **42 push_swap** project as a `no_std` Rust binary (rustc only, no Cargo). Model APIs are served by **OpenRouter**. A Python script on the host orchestrates one container per `(model, run)` pair, collects the generated code, builds it, runs a Python test harness against the resulting binary, and prints a unified table.

Inspired by [`akitaonrails/llm-coding-benchmark`](https://github.com/akitaonrails/llm-coding-benchmark) — simpler: one phase, one task, no Docker-in-Docker, no LLM-as-judge.

## Requirements

- Docker (host-side, for the opencode container and the validator container)
- Python 3.10+
- An OpenRouter API key (`OPENROUTER_API_KEY`)

## Quickstart

```sh
# 1. Build the opencode image once (Alpine + rustc + gcc + lldb + ...)
make image

# 2. Set your OpenRouter key
cp .env.example .env
$EDITOR .env

# 3. Run the benchmark + validation in one shot
make run
```

Other Make targets: `make validate` (re-validate without re-running benchmarks), `make clean` (delete `results/`), `make re` (clean + run), `make status` (show running benchmark containers), `make help`.

Output lands in `results/<slug>-run-<n>/`:

```
results/
├── claude-opus-4-6-run-0/
│   ├── code/             # generated source (Makefile, push_swap.rs) + the built binary
│   ├── result.json       # tokens, cost, elapsed, exit_code, file census
│   ├── validation.json   # per-test pass/fail from test_push_swap.py
│   ├── opencode.ndjson   # raw stdout (debugging)
│   └── stderr.log
├── claude-sonnet-4-6-run-0/
├── ...
├── summary.json
└── validation_summary.json
```

## Validating the generated code

`make run` validates automatically at the end. To re-validate without re-running the benchmark, use `make validate`.

`validate.py` walks every `results/<slug>-run-N/` directory and:

1. Builds `code/push_swap` inside docker if missing (`make`).
2. Mounts the binary into a fresh container and runs `validation/test_push_swap.py /push_swap`.
3. The Python tester runs the binary against ~20 inputs (sorted, descending, 100/500-element random permutations, malformed args, etc.), parses the printed operation stream from stdout, replays each op on a simulated pair of stacks, and asserts the result equals `sorted(input)` with stack b empty.
4. Per-test results are written in Unity-format lines (`file:line:test_name:PASS|FAIL`) so the existing parser turns them into `validation.json`.

To validate a single binary manually:

```sh
docker run --rm \
  -v $(pwd)/validation:/validation \
  -v $(pwd)/results/claude-opus-4-6-run-0/code/push_swap:/push_swap:ro \
  -w /validation \
  ai-benchmark-opencode \
  python3 test_push_swap.py /push_swap
```

## Configuration

All knobs live in [`benchmark-config.json`](./benchmark-config.json) — `benchmark.py` takes no CLI flags. Edit the file, re-run the script.

| field | meaning |
|---|---|
| `image` | Docker image tag built from `Dockerfile` |
| `runs_per_model` | independent containers per model |
| `timeout_seconds` | wall-clock cap per run |
| `results_dir` | where per-run output trees go |
| `prompt_file` | the markdown prompt fed to opencode |
| `opencode_config_path` | mounted read-only at `/work/opencode.json` |
| `models[]` | `{ slug, model, label }` — `model` is the exact OpenRouter id |

Per-model reasoning effort is configured in `opencode.json` under `provider.openrouter.models.<id>.options.reasoning.effort` (low/medium/high).

### Resume

Re-running skips any `<slug>-run-<n>/` whose `result.json` reports `exit_code == 0` and `timed_out == false`. Force a re-run by deleting that directory.

## How it works

For each `(model, run_idx)`:

1. `prompt.md` is copied verbatim into `results/<slug>-run-<n>/prompt.txt`.
2. The orchestrator spawns a docker container running `opencode run --format json -m openrouter/<model> "$(cat /prompt.txt)"`. Stdout streams to `opencode.ndjson` and is parsed line-by-line for a live ticker.
3. The model writes generated files into the bind-mounted `code/` directory.
4. `result.json` is written with: timestamps, elapsed seconds, exit code, timeout flag, token totals + session id + finish reason + cumulative cost parsed from the NDJSON, and a file census.
5. After all runs, `validate.run_all()` builds + tests each artifact, and `validate.print_table()` prints the combined results table.

Sequential execution — kinder to OpenRouter rate limits and avoids races on stdout buffers.

## Files

```
.
├── benchmark-config.json    # single source of truth for the orchestrator
├── benchmark.py             # orchestrator (no CLI flags)
├── validate.py              # walks results/, builds and tests each push_swap
├── prompt.md                # the prompt fed to opencode (push_swap, no_std Rust)
├── opencode.json            # opencode provider config (OpenRouter, per-model reasoning)
├── Dockerfile               # alpine + rustc + gcc + opencode
├── Makefile                 # make image / run / validate / status / clean
├── validation/
│   ├── Makefile             # tiny wrapper that runs the Python tester
│   └── test_push_swap.py    # spec-compliant push_swap tester (Unity-format output)
└── results/                 # gitignored
```

## What this benchmark deliberately does NOT do

- No second validation phase via opencode (the model is not asked to compile/test through the agent).
- No Docker-in-Docker. The image carries `rustc` + `make` + `gcc` so the model can compile during generation; validation runs in its own short-lived container on the host.
- No LLM-as-judge. `validation.json` records mechanical pass/fail; ranking is up to you.
- No retry on OpenRouter errors. A failed run gets non-zero `exit_code` and the sweep moves on.
