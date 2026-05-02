#!/usr/bin/env python3
"""Validate a push_swap binary.

For each test case, runs the binary, parses operations from stdout, replays
them on a simulated pair of stacks, asserts the result. Emits Unity-format
lines so validate.py's existing parser keeps working.

Usage:
    python3 test_push_swap.py /path/to/push_swap
"""

from __future__ import annotations

import random
import subprocess
import sys
from pathlib import Path

VALID_OPS = {"sa", "sb", "ss", "pa", "pb", "ra", "rb", "rr", "rra", "rrb", "rrr"}


def simulate(initial_a: list[int], ops: list[str]) -> tuple[list[int], list[int], str | None]:
    a = list(initial_a)
    b: list[int] = []
    for op in ops:
        if op not in VALID_OPS:
            return a, b, f"unknown op {op!r}"
        if op == "sa":
            if len(a) >= 2:
                a[0], a[1] = a[1], a[0]
        elif op == "sb":
            if len(b) >= 2:
                b[0], b[1] = b[1], b[0]
        elif op == "ss":
            if len(a) >= 2:
                a[0], a[1] = a[1], a[0]
            if len(b) >= 2:
                b[0], b[1] = b[1], b[0]
        elif op == "pa":
            if b:
                a.insert(0, b.pop(0))
        elif op == "pb":
            if a:
                b.insert(0, a.pop(0))
        elif op == "ra":
            if len(a) >= 2:
                a.append(a.pop(0))
        elif op == "rb":
            if len(b) >= 2:
                b.append(b.pop(0))
        elif op == "rr":
            if len(a) >= 2:
                a.append(a.pop(0))
            if len(b) >= 2:
                b.append(b.pop(0))
        elif op == "rra":
            if len(a) >= 2:
                a.insert(0, a.pop())
        elif op == "rrb":
            if len(b) >= 2:
                b.insert(0, b.pop())
        elif op == "rrr":
            if len(a) >= 2:
                a.insert(0, a.pop())
            if len(b) >= 2:
                b.insert(0, b.pop())
    return a, b, None


def run_binary(binary: Path, args: list[str]) -> tuple[int, str, str]:
    proc = subprocess.run(
        [str(binary)] + args,
        capture_output=True, text=True, timeout=20,
    )
    return proc.returncode, proc.stdout, proc.stderr


def emit(name: str, status: str, msg: str = "") -> None:
    """Unity-format line — file:line:test:status[:message]."""
    line = f"test_push_swap.py:0:{name}:{status}"
    if msg:
        line += f": {msg}"
    print(line)


# ----------------- test cases -----------------

class Result:
    def __init__(self) -> None:
        self.tests = 0
        self.failures = 0

    def record(self, name: str, ok: bool, msg: str = "") -> None:
        self.tests += 1
        if ok:
            emit(name, "PASS")
        else:
            self.failures += 1
            emit(name, "FAIL", msg)


def test_correctness(res: Result, name: str, binary: Path, args: list[int],
                     max_ops: int | None = None, metric: str | None = None) -> None:
    str_args = [str(x) for x in args]
    try:
        rc, out, err = run_binary(binary, str_args)
    except subprocess.TimeoutExpired:
        res.record(name, False, "binary timed out")
        return
    if rc != 0:
        res.record(name, False, f"exit={rc} stderr={err.strip()[:80]!r}")
        return
    ops = out.split()
    for op in ops:
        if op not in VALID_OPS:
            res.record(name, False, f"invalid op {op!r}")
            return
    final_a, final_b, sim_err = simulate(args, ops)
    if sim_err:
        res.record(name, False, sim_err)
        return
    if final_a != sorted(args):
        res.record(name, False, f"a not sorted (len={len(args)}, ops={len(ops)})")
        return
    if final_b:
        res.record(name, False, f"b not empty (n={len(final_b)})")
        return
    if max_ops is not None and len(ops) > max_ops:
        res.record(name, False, f"ops={len(ops)} > max={max_ops}")
        return
    res.record(name, True)
    if metric:
        # Picked up by validate.py's parse_unity() and surfaced in the table.
        print(f"# METRIC {metric}={len(ops)}")


def test_no_output(res: Result, name: str, binary: Path, args: list[str]) -> None:
    try:
        rc, out, err = run_binary(binary, args)
    except subprocess.TimeoutExpired:
        res.record(name, False, "timeout")
        return
    if rc != 0:
        res.record(name, False, f"exit={rc}")
        return
    if out.strip() != "":
        res.record(name, False, f"unexpected stdout: {out.strip()[:60]!r}")
        return
    res.record(name, True)


def test_error(res: Result, name: str, binary: Path, args: list[str]) -> None:
    try:
        rc, out, err = run_binary(binary, args)
    except subprocess.TimeoutExpired:
        res.record(name, False, "timeout")
        return
    if rc == 0:
        res.record(name, False, "expected non-zero exit on bad input")
        return
    if "Error" not in err:
        res.record(name, False, f"expected 'Error' on stderr; got {err.strip()[:60]!r}")
        return
    if out.strip() != "":
        res.record(name, False, f"unexpected stdout: {out.strip()[:60]!r}")
        return
    res.record(name, True)


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: test_push_swap.py <push_swap binary>", file=sys.stderr)
        return 2
    binary = Path(sys.argv[1])
    if not binary.is_file():
        print(f"binary not found: {binary}", file=sys.stderr)
        return 2

    res = Result()
    rng = random.Random(0xC0FFEE)

    # No-op cases (no output expected)
    test_no_output(res, "test_empty",          binary, [])
    test_no_output(res, "test_single",         binary, ["42"])
    test_no_output(res, "test_already_sorted", binary, ["1", "2", "3", "4", "5"])

    # Small correctness
    test_correctness(res, "test_two_unsorted",       binary, [2, 1])
    test_correctness(res, "test_three_descending",   binary, [3, 2, 1], max_ops=5)
    test_correctness(res, "test_three_mixed",        binary, [2, 3, 1], max_ops=5)
    test_correctness(res, "test_three_negatives",    binary, [-1, -3, -2], max_ops=5)
    test_correctness(res, "test_five_random",        binary, [4, 1, 5, 2, 3], max_ops=16)
    test_correctness(res, "test_five_descending",    binary, [5, 4, 3, 2, 1], max_ops=16)

    # Larger random sets — also emit op counts as metrics for the table
    nums_100 = rng.sample(range(-10000, 10000), 100)
    test_correctness(res, "test_100_random", binary, nums_100, max_ops=1500, metric="ops_100")

    nums_500 = rng.sample(range(-100000, 100000), 500)
    test_correctness(res, "test_500_random", binary, nums_500, max_ops=11500, metric="ops_500")

    # Quoted-string single-arg form: ./push_swap "3 1 4 5 9 2 6"
    quoted_input = [3, 1, 4, 5, 9, 2, 6]
    try:
        rc, out, _ = run_binary(binary, [" ".join(str(x) for x in quoted_input)])
        ops = out.split()
        final_a, final_b, sim_err = simulate(quoted_input, ops)
        ok = rc == 0 and not sim_err and final_a == sorted(quoted_input) and not final_b
        res.record("test_quoted_string_form", ok,
                   "" if ok else f"rc={rc} sim_err={sim_err} a={final_a[:5]} b={final_b[:5]}")
    except subprocess.TimeoutExpired:
        res.record("test_quoted_string_form", False, "timeout")

    # Error cases
    test_error(res, "test_error_non_numeric",   binary, ["1", "abc", "3"])
    test_error(res, "test_error_duplicate",     binary, ["1", "2", "3", "2"])
    test_error(res, "test_error_overflow",      binary, ["99999999999"])
    test_error(res, "test_error_empty_arg",     binary, [""])

    # Unity-style totals line
    print(f"\n{res.tests} Tests {res.failures} Failures 0 Ignored")
    print("OK" if res.failures == 0 else "FAIL")
    return 0 if res.failures == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
