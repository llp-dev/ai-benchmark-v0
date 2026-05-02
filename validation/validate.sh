#!/usr/bin/env bash
# Validate one model's push_swap implementation.
#
# Required env:
#   WORK     path to model's work directory (host path)
# Optional env:
#   VALIDATE_IMAGE  docker image with rust + make + gcompat (default: ai-benchmark-validate)
#
# Effects:
#   - Rebuilds push_swap inside the validate image (`make re`).
#   - Runs the binary against a fixed list of inputs.
#   - Pipes each output to checker_linux to confirm it sorts.
#   - Writes <work>/../outputs/validation.json with {pass, fail, total, build_ok, cases}.
#   - Exits non-zero iff any case fails or the build fails.

set -euo pipefail

WORK="${WORK:?WORK env var (path to model work dir) required}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CHECKER="${SCRIPT_DIR}/checker_linux"
IMAGE="${VALIDATE_IMAGE:-ai-benchmark-validate}"

[[ -x "$CHECKER" ]] || { echo "missing checker binary: $CHECKER" >&2; exit 1; }
[[ -d "$WORK"   ]] || { echo "missing work dir: $WORK" >&2; exit 1; }

WORK_ABS="$(cd "$WORK" && pwd)"
OUTPUTS_ABS="$(cd "$WORK/.." && pwd)/outputs"
mkdir -p "$OUTPUTS_ABS"

docker run --rm -i \
    -v "$WORK_ABS:/work" \
    -v "$OUTPUTS_ABS:/outputs" \
    -v "$CHECKER:/checker:ro" \
    -w /work \
    "$IMAGE" \
    sh -s <<'EOSH'
set -u

write_json() {
    # $1=build_ok, $2=pass, $3=fail, $4=cases_json
    cat > /outputs/validation.json <<EOJ
{"build_ok":$1,"pass":$2,"fail":$3,"total":$(($2 + $3)),"cases":$4}
EOJ
}

echo "[build] make re"
if ! make re; then
    echo "[build] FAILED"
    write_json false 0 0 "[]"
    exit 1
fi

PASS=0
FAIL=0
CASES=""

run_case() {
    local label="$1"; shift
    local ops result n_ops status reason

    if ! ops="$(./push_swap "$@" 2>&1)"; then
        FAIL=$((FAIL + 1))
        printf "  FAIL [%-22s] push_swap exec failed\n" "$label"
        CASES="${CASES}{\"label\":\"$label\",\"status\":\"fail\",\"reason\":\"exec\"},"
        return
    fi
    n_ops=$(printf '%s' "$ops" | grep -c . || true)
    if [ -z "$ops" ]; then
        result="$(/checker "$@" </dev/null 2>/dev/null || echo ERR)"
    else
        result="$(printf '%s\n' "$ops" | /checker "$@" 2>/dev/null || echo ERR)"
    fi
    if [ "$result" = "OK" ]; then
        PASS=$((PASS + 1)); status="ok"; reason=""
        printf "  ok   [%-22s] ops=%d\n" "$label" "$n_ops"
    else
        FAIL=$((FAIL + 1)); status="fail"; reason="$result"
        printf "  FAIL [%-22s] checker=%s ops=%d\n" "$label" "$result" "$n_ops"
    fi
    CASES="${CASES}{\"label\":\"$label\",\"status\":\"$status\",\"ops\":$n_ops,\"reason\":\"$reason\"},"
}

echo "[test ] running cases"
run_case "single"           42
run_case "two-sorted"       1 2
run_case "two-reverse"      2 1
run_case "three-sorted"     1 2 3
run_case "three-reverse"    3 2 1
run_case "three-mixed"      2 1 3
run_case "five-reverse"     5 4 3 2 1
run_case "five-mixed"       3 1 4 5 2
run_case "negatives"        -5 0 5 -3 7
run_case "large-spread"     1000 -2000 0 500 -100
run_case "n=10-reverse"     $(seq 10 -1 1)
run_case "n=100-reverse"    $(seq 100 -1 1)
run_case "n=100-shuffled"   $(seq 1 100 | awk 'BEGIN{srand(42)} {print rand()"\t"$0}' | sort | cut -f2)
run_case "n=500-reverse"    $(seq 500 -1 1)
run_case "n=500-shuffled"   $(seq 1 500 | awk 'BEGIN{srand(7)}  {print rand()"\t"$0}' | sort | cut -f2)

echo "[done] PASS=$PASS FAIL=$FAIL"
write_json true "$PASS" "$FAIL" "[${CASES%,}]"
[ "$FAIL" -eq 0 ]
EOSH
