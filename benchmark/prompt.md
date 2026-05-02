# push_swap (Rust binary, restricted std) — Benchmark Prompt

## Task

Implement the **42 push_swap** project in **Rust**. Output is a single executable named `push_swap` that takes a list of integers as command-line arguments and prints a sequence of stack operations that sort those integers, using only the allowed instruction set. Build with `rustc` directly — **no Cargo, no `Cargo.toml`, no crates.io dependencies**. The Rust standard library (`std`) is available but with the restrictions in [Allowed / forbidden imports](#allowed--forbidden-imports) below.

## How push_swap works

You manage two stacks, `a` and `b`. Stack `a` starts containing the input integers (top of the stack = first argument). Stack `b` starts empty. Your job is to print operations that, when applied in order, leave stack `a` sorted ascending (smallest on top) with stack `b` empty.

### Instruction set (case-sensitive, one per line, lowercase)

| op   | effect |
|------|--------|
| `sa` | swap the top two elements of stack a (does nothing if a has < 2 elements) |
| `sb` | swap the top two elements of stack b |
| `ss` | `sa` and `sb` together |
| `pa` | take the top element of b and push it onto a (does nothing if b is empty) |
| `pb` | take the top element of a and push it onto b (does nothing if a is empty) |
| `ra` | rotate a up: the first element becomes the last (does nothing if a has < 2) |
| `rb` | rotate b up: same on b |
| `rr` | `ra` and `rb` together |
| `rra`| reverse rotate a: the last element becomes the first |
| `rrb`| reverse rotate b: same on b |
| `rrr`| `rra` and `rrb` together |

Each operation must be printed on its own line, in lowercase, with no leading/trailing whitespace.

### Argument parsing rules

- The integers may be passed as **separate arguments** (`./push_swap 3 1 4 1 5`) or as **one quoted string** (`./push_swap "3 1 4 1 5"`). Both forms must work; treat any whitespace-delimited tokens inside an argument as separate integers.
- Each token must parse as a 32-bit signed integer. An optional leading `+` or `-` is allowed; otherwise digits only.
- Duplicates are not allowed.
- If the input is empty, print nothing and exit 0.
- If the input is already sorted ascending, print nothing and exit 0.
- On any input error (non-integer, overflow beyond i32 range, duplicate), print `Error\n` to **stderr** and exit with non-zero status. Print nothing to stdout.
- An argv slot that is the empty string (e.g. `./push_swap ""`) is malformed input — print `Error\n` to **stderr** and exit non-zero.

### Operation count grading

The harness enforces only the passing floor on each test size, but the actual op count is recorded so you can be ranked across the 42 subject's grading bands:

| input         | grading bands (best → passing floor)                              |
| ---           | ---                                                               |
| 3 elements    | ≤ 3 (excellent) · ≤ 5 (passing floor)                             |
| 5 elements    | ≤ 12 (excellent) · ≤ 16 (passing floor)                           |
| 100 random    | ≤ 700 / ≤ 900 / ≤ 1100 / ≤ 1300 / ≤ 1500 (passing floor)          |
| 500 random    | ≤ 5500 / ≤ 7000 / ≤ 8500 / ≤ 10000 / ≤ 11500 (passing floor)      |

Hitting ≤ 1500 / ≤ 11500 is a pass. Lower is meaningfully better — the gap between 700 and 1500 ops on n=100 is the difference between a chunked / radix-style algorithm and a quadratic one.

## Working directory contract

- Working directory is `/work`. Write all files at the root.
- No subdirectories, no `Cargo.toml`, no `Cargo.lock`, no `.cargo/`.
- Do not modify or remove `opencode.json` if you see it.

## Deliverables (at `/work` root)

1. `push_swap.rs` — single Rust source file with a regular `fn main() { ... }` entry point.
2. `Makefile` — builds `push_swap` from `push_swap.rs`.

## Build contract

`make` (or `make all`) must produce the executable `push_swap` at the root of `/work`. The compile rule must be exactly:

```
rustc --edition 2021 --crate-type bin -C opt-level=2 -D warnings -o push_swap push_swap.rs
```

Makefile rules required: `all`, `$(NAME)`, `clean`, `fclean`, `re`. No unnecessary relinking. Zero warnings (`-D warnings` is mandatory).

## Allowed / forbidden imports

The Rust standard library is available, but the surface you may use is deliberately tiny. Anything else you need, you build yourself on top of these.

### Allowed

| use case | Rust item |
|---|---|
| storage          | `Vec<T>` and its methods (constructors, `push`, `pop`, `insert`, `remove`, `swap`, `len`, indexing, `iter`, `iter_mut`, `truncate`, `clear`) — `Vec` is your sole container |
| exit code        | `std::process::exit` |
| read argv        | `std::env::args` |
| stdout output    | `println!` |
| stderr output    | `eprintln!` |
| integer parsing  | `i32::from_str_radix`, `<&str as core::str::FromStr>::parse::<i32>()` |

Plus: primitive `&str` / `String` methods (split / trim / chars / bytes / etc.), basic control flow, integer arithmetic, and any helper functions / types **you write yourself** in `push_swap.rs`.

### Forbidden

- **No other output path**. Specifically banned: `print!`, `eprint!`, `write!`, `writeln!`, `format!`, `std::fmt::*`, `std::io::Write`, `std::io::BufWriter`, `std::io::stdout()`, `std::io::stderr()`. Use `println!` / `eprintln!` exclusively, with literal-string formats — e.g. `println!("ra")`, `eprintln!("Error")`.
- **No `VecDeque`** and no other container from `std::collections::*` — `BinaryHeap`, `BTreeMap`, `BTreeSet`, `HashMap`, `HashSet`. `Vec` is the only container. Build any deque / ring-buffer semantics yourself.
- **No `Box`, no `Rc`, no `Arc`, no `RefCell`, no `Cell`.** No alternative heap allocations.
- **No sort helpers**: `<[T]>::sort`, `sort_unstable`, `sort_by`, `sort_by_key`, `sort_unstable_by`, `sort_unstable_by_key`, `Vec::sort*`. Deriving the sort is the entire task.
- **No external crates**, no `Cargo.toml`, no `Cargo.lock`, no `.cargo/`.
- No `core::ptr::copy*`, `core::ptr::write_bytes`, `core::slice::from_raw_parts*`, `core::intrinsics::*`.
- No `unsafe` blocks. Stay in safe Rust.
- No global mutable state, no `static mut`.

### Other rules

- The program must not panic on the documented inputs (no out-of-bounds indexing, no `unwrap()` on values that may legitimately be `None`, no overflow).
- Output operations go to **stdout**; errors go to **stderr**.

> The harness runs a static scan over `push_swap.rs` (after stripping comments and string literals) and reports any forbidden pattern as a **violation**, alongside the test results. Don't try to launder banned items through aliases or `use ... as`; the scan checks the raw text.

## Verification (the host does this automatically)

The host runs `push_swap` on a battery of inputs, captures stdout, replays the operations on a simulated pair of stacks, and asserts that the final state of stack `a` equals the input sorted ascending and that stack `b` is empty. Test cases include:

- empty argv
- already sorted lists of various sizes
- the smallest non-trivial cases (2, 3, 5 numbers)
- random permutations of 100 and 500 distinct integers
- duplicate / non-numeric / overflowing inputs (must exit non-zero with `Error\n` on stderr)

Stop when `make` succeeds, `./push_swap` runs without crashing, and the output of `./push_swap 3 1 2` is a valid sorting sequence.
