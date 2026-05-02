#!/usr/bin/env python3
import subprocess
import sys

def simulate_ops(initial, ops):
    a = list(initial)
    b = []
    
    for op in ops:
        op = op.strip()
        if op == 'sa':
            if len(a) >= 2:
                a[0], a[1] = a[1], a[0]
        elif op == 'sb':
            if len(b) >= 2:
                b[0], b[1] = b[1], b[0]
        elif op == 'ss':
            if len(a) >= 2:
                a[0], a[1] = a[1], a[0]
            if len(b) >= 2:
                b[0], b[1] = b[1], b[0]
        elif op == 'pa':
            if b:
                a.insert(0, b.pop(0))
        elif op == 'pb':
            if a:
                b.insert(0, a.pop(0))
        elif op == 'ra':
            if len(a) >= 2:
                a.append(a.pop(0))
        elif op == 'rb':
            if len(b) >= 2:
                b.append(b.pop(0))
        elif op == 'rr':
            if len(a) >= 2:
                a.append(a.pop(0))
            if len(b) >= 2:
                b.append(b.pop(0))
        elif op == 'rra':
            if len(a) >= 2:
                a.insert(0, a.pop())
        elif op == 'rrb':
            if len(b) >= 2:
                b.insert(0, b.pop())
        elif op == 'rrr':
            if len(a) >= 2:
                a.insert(0, a.pop())
            if len(b) >= 2:
                b.insert(0, b.pop())
    
    return a, b

def test(args_str, expected_sorted=True):
    args = args_str.split()
    result = subprocess.run(['./push_swap'] + args, capture_output=True, text=True)
    
    if result.stderr:
        print(f"  stderr: {result.stderr.strip()}")
        return result.returncode != 0
    
    ops = result.stdout.strip().split('\n') if result.stdout.strip() else []
    initial = [int(x) for x in args]
    a, b = simulate_ops(initial, ops)
    
    if expected_sorted:
        expected = sorted(initial)
        if a == expected and b == []:
            print(f"  PASS ({len(ops)} ops)")
            return True
        else:
            print(f"  FAIL: a={a}, b={b}, expected a={expected}")
            return False
    return True

print("Test: empty")
test("")

print("Test: already sorted")
test("1 2 3")

print("Test: 2 elements")
test("2 1")

print("Test: 3 elements")
test("3 1 2")

print("Test: 5 elements")
test("5 4 3 2 1")

print("Test: 5 elements random")
test("3 1 5 2 4")

import random
for size in [100, 500]:
    for trial in range(3):
        nums = random.sample(range(-10000, 10000), size)
        args_str = ' '.join(str(x) for x in nums)
        print(f"Test: {size} elements (trial {trial+1})")
        test(args_str)

print("Test: duplicate")
test("1 1")

print("Test: non-integer")
test("abc")

print("Test: overflow")
test("2147483648")

print("Done")
