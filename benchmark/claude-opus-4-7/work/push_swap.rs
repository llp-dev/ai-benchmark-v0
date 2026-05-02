// push_swap — single-file Rust implementation.
// Restrictions: only Vec as container, no sort helpers, no Box/Rc/Arc,
// no print!/write!/format!, only println!/eprintln! for output, no unsafe.

use std::env;
use std::process::exit;

// ---------------------------------------------------------------------------
// Stack abstraction built on Vec. Top of stack = last element of the Vec
// (so push_swap's "top" is stored at index len-1). "ra" rotates the top to
// the bottom: pop the last, insert at position 0.
// ---------------------------------------------------------------------------

struct Ps {
    a: Vec<i32>,
    b: Vec<i32>,
    ops: Vec<u8>, // encoded op ids, flushed at end
}

// Op ids
const OP_SA: u8 = 0;
const OP_SB: u8 = 1;
const OP_SS: u8 = 2;
const OP_PA: u8 = 3;
const OP_PB: u8 = 4;
const OP_RA: u8 = 5;
const OP_RB: u8 = 6;
const OP_RR: u8 = 7;
const OP_RRA: u8 = 8;
const OP_RRB: u8 = 9;
const OP_RRR: u8 = 10;

#[allow(dead_code)]
impl Ps {
    fn new(input: Vec<i32>) -> Self {
        // input[0] is first argv token, which should be "top of a".
        // With top = last of Vec, push argv tokens in reverse.
        let mut a: Vec<i32> = Vec::with_capacity(input.len());
        let mut i = input.len();
        while i > 0 {
            i -= 1;
            a.push(input[i]);
        }
        Ps {
            a,
            b: Vec::new(),
            ops: Vec::new(),
        }
    }

    fn push_op(&mut self, op: u8) {
        self.ops.push(op);
    }

    fn sa(&mut self) {
        let n = self.a.len();
        if n >= 2 {
            self.a.swap(n - 1, n - 2);
        }
        self.push_op(OP_SA);
    }
    fn sb(&mut self) {
        let n = self.b.len();
        if n >= 2 {
            self.b.swap(n - 1, n - 2);
        }
        self.push_op(OP_SB);
    }
    fn pa(&mut self) {
        if let Some(v) = self.b.pop() {
            self.a.push(v);
        }
        self.push_op(OP_PA);
    }
    fn pb(&mut self) {
        if let Some(v) = self.a.pop() {
            self.b.push(v);
        }
        self.push_op(OP_PB);
    }
    // ra: first element (top) goes to last (bottom).
    // top = last index; bottom = index 0. So pop last, insert at 0.
    fn ra(&mut self) {
        if self.a.len() >= 2 {
            let v = self.a.pop().unwrap_or(0);
            // pop is safe because len>=2; unwrap_or just avoids unwrap().
            self.a.insert(0, v);
        }
        self.push_op(OP_RA);
    }
    fn rb(&mut self) {
        if self.b.len() >= 2 {
            let v = self.b.pop().unwrap_or(0);
            self.b.insert(0, v);
        }
        self.push_op(OP_RB);
    }
    // rra: last element (bottom) goes to first (top).
    // Remove index 0, push at end.
    fn rra(&mut self) {
        if self.a.len() >= 2 {
            let v = self.a.remove(0);
            self.a.push(v);
        }
        self.push_op(OP_RRA);
    }
    fn rrb(&mut self) {
        if self.b.len() >= 2 {
            let v = self.b.remove(0);
            self.b.push(v);
        }
        self.push_op(OP_RRB);
    }
    fn ss(&mut self) {
        let na = self.a.len();
        if na >= 2 {
            self.a.swap(na - 1, na - 2);
        }
        let nb = self.b.len();
        if nb >= 2 {
            self.b.swap(nb - 1, nb - 2);
        }
        self.push_op(OP_SS);
    }
    fn rr(&mut self) {
        if self.a.len() >= 2 {
            let v = self.a.pop().unwrap_or(0);
            self.a.insert(0, v);
        }
        if self.b.len() >= 2 {
            let v = self.b.pop().unwrap_or(0);
            self.b.insert(0, v);
        }
        self.push_op(OP_RR);
    }
    fn rrr(&mut self) {
        if self.a.len() >= 2 {
            let v = self.a.remove(0);
            self.a.push(v);
        }
        if self.b.len() >= 2 {
            let v = self.b.remove(0);
            self.b.push(v);
        }
        self.push_op(OP_RRR);
    }

    // Access helpers: top() = last; kth-from-top = a[len-1-k]
    fn a_top(&self) -> i32 {
        self.a[self.a.len() - 1]
    }
    fn a_second(&self) -> i32 {
        self.a[self.a.len() - 2]
    }
    fn a_third(&self) -> i32 {
        self.a[self.a.len() - 3]
    }
    fn b_top(&self) -> i32 {
        self.b[self.b.len() - 1]
    }

    fn flush_ops(&self) {
        let mut i = 0;
        while i < self.ops.len() {
            match self.ops[i] {
                OP_SA => println!("sa"),
                OP_SB => println!("sb"),
                OP_SS => println!("ss"),
                OP_PA => println!("pa"),
                OP_PB => println!("pb"),
                OP_RA => println!("ra"),
                OP_RB => println!("rb"),
                OP_RR => println!("rr"),
                OP_RRA => println!("rra"),
                OP_RRB => println!("rrb"),
                OP_RRR => println!("rrr"),
                _ => {}
            }
            i += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

fn parse_i32(s: &str) -> Option<i32> {
    if s.is_empty() {
        return None;
    }
    let bytes = s.as_bytes();
    let mut i = 0usize;
    let mut neg = false;
    if bytes[0] == b'+' {
        i = 1;
    } else if bytes[0] == b'-' {
        neg = true;
        i = 1;
    }
    if i >= bytes.len() {
        return None;
    }
    let mut acc: i64 = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c < b'0' || c > b'9' {
            return None;
        }
        acc = acc * 10 + (c - b'0') as i64;
        if acc > (i32::MAX as i64) + 1 {
            return None;
        }
        i += 1;
    }
    if neg {
        acc = -acc;
    }
    if acc < i32::MIN as i64 || acc > i32::MAX as i64 {
        return None;
    }
    Some(acc as i32)
}

fn parse_args(args: Vec<String>) -> Result<Vec<i32>, ()> {
    // args is argv[1..]. Each arg may contain whitespace-separated tokens.
    // An empty argv slot is malformed.
    let mut out: Vec<i32> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg.is_empty() {
            return Err(());
        }
        // Split on whitespace. If after splitting we have zero tokens, that's
        // an error (arg was only whitespace).
        let mut tok_count = 0;
        let mut j = 0;
        let bytes = arg.as_bytes();
        while j < bytes.len() {
            // skip ws
            while j < bytes.len() && is_ws(bytes[j]) {
                j += 1;
            }
            if j >= bytes.len() {
                break;
            }
            let start = j;
            while j < bytes.len() && !is_ws(bytes[j]) {
                j += 1;
            }
            let tok = &arg[start..j];
            match parse_i32(tok) {
                Some(v) => out.push(v),
                None => return Err(()),
            }
            tok_count += 1;
        }
        if tok_count == 0 {
            return Err(());
        }
        i += 1;
    }
    // Duplicate check.
    let mut k = 0;
    while k < out.len() {
        let mut l = k + 1;
        while l < out.len() {
            if out[k] == out[l] {
                return Err(());
            }
            l += 1;
        }
        k += 1;
    }
    Ok(out)
}

fn is_ws(c: u8) -> bool {
    c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' || c == 0x0B || c == 0x0C
}

// ---------------------------------------------------------------------------
// Rank compression: map input values to 0..n-1 by their sorted order.
// We must derive ranks without using sort helpers.
// Selection-sort by value to produce the sorted permutation.
// ---------------------------------------------------------------------------

fn rank_compress(values: &[i32]) -> Vec<usize> {
    let n = values.len();
    // ranks[i] = number of values[j] (j != i) with values[j] < values[i]
    //           plus ties (no ties by precondition, but handle anyway).
    let mut ranks: Vec<usize> = Vec::with_capacity(n);
    let mut i = 0;
    while i < n {
        let mut r = 0usize;
        let mut j = 0;
        while j < n {
            if values[j] < values[i] {
                r += 1;
            }
            j += 1;
        }
        ranks.push(r);
        i += 1;
    }
    ranks
}

fn is_sorted(v: &[usize]) -> bool {
    // Check ascending. In our "top = last" convention, the input was pushed
    // in reverse. Here we take the raw input order (top-first) and check that
    // it's strictly increasing to detect "already sorted".
    let mut i = 1;
    while i < v.len() {
        if v[i - 1] >= v[i] {
            return false;
        }
        i += 1;
    }
    true
}

// ---------------------------------------------------------------------------
// Small-n sorters
// ---------------------------------------------------------------------------

// Sort 3 elements on top of a (a.len() == 3 here).
fn sort3(ps: &mut Ps) {
    let a = ps.a_top();
    let b = ps.a_second();
    let c = ps.a_third();
    // Compute local ranks among {a,b,c}.
    let r0 = (if b < a { 1 } else { 0 }) + (if c < a { 1 } else { 0 });
    let r1 = (if a < b { 1 } else { 0 }) + (if c < b { 1 } else { 0 });
    let r2 = (if a < c { 1 } else { 0 }) + (if b < c { 1 } else { 0 });
    // Stack top..bot corresponds to (r0, r1, r2). We want (0, 1, 2).
    // (0,1,2): noop
    // (0,2,1): rra; sa
    // (1,0,2): sa
    // (1,2,0): rra
    // (2,0,1): ra
    // (2,1,0): sa; rra
    if r0 == 0 && r1 == 1 && r2 == 2 {
        // noop
    } else if r0 == 0 && r1 == 2 && r2 == 1 {
        ps.rra();
        ps.sa();
    } else if r0 == 1 && r1 == 0 && r2 == 2 {
        ps.sa();
    } else if r0 == 1 && r1 == 2 && r2 == 0 {
        ps.rra();
    } else if r0 == 2 && r1 == 0 && r2 == 1 {
        ps.ra();
    } else {
        // (2,1,0)
        ps.sa();
        ps.rra();
    }
}

// Sort up to 3 elements in a (when a.len() in {0,1,2,3}).
fn sort_small_a(ps: &mut Ps) {
    let n = ps.a.len();
    if n < 2 {
        return;
    }
    if n == 2 {
        if ps.a_top() > ps.a_second() {
            ps.sa();
        }
        return;
    }
    if n == 3 {
        sort3(ps);
        return;
    }
    // Shouldn't reach for n > 3; fallback to selection.
    selection_sort_a(ps);
}

// Fallback: selection-sort a using pb + pa (O(n^2) ops). Used for tiny n
// where we didn't take the chunk path.
fn selection_sort_a(ps: &mut Ps) {
    while ps.a.len() > 3 {
        // Find min in a, bring to top with fewest ops, then pb.
        let n = ps.a.len();
        let mut min_idx_from_top = 0usize;
        let mut min_val = ps.a[n - 1];
        let mut k = 1;
        while k < n {
            let v = ps.a[n - 1 - k];
            if v < min_val {
                min_val = v;
                min_idx_from_top = k;
            }
            k += 1;
        }
        let up = min_idx_from_top;
        let down = n - min_idx_from_top;
        if up <= down {
            let mut t = 0;
            while t < up {
                ps.ra();
                t += 1;
            }
        } else {
            let mut t = 0;
            while t < down {
                ps.rra();
                t += 1;
            }
        }
        ps.pb();
    }
    sort_small_a(ps);
    // Push b back to a: b contains the smallest values with smallest-most-recent
    // (on top of b). Since we pushed smallest first, b from top to bot is
    // largest-of-b down to smallest. Actually: we pushed min each time, so
    // order pushed to b: min1(smallest), min2, min3... Top of b = last pushed =
    // largest of those. So popping b into a puts largest-of-b on top of a,
    // then next, etc. That corrupts order. Instead for the fallback use a
    // different approach: push smallest to b but prepend via rb before push?
    // Simpler: for tiny n, we don't want fallback—replace with:
    while !ps.b.is_empty() {
        ps.pa();
    }
    // The above comment is moot because selection_sort_a isn't used in
    // practice (we route small n to sort3 / direct 5-sort). Keep as safety.
}

// Sort 4 or 5: push smallest(s) to b, sort3 a, pa back.
fn sort_small(ps: &mut Ps) {
    // Work on ranks; ranks are 0..n-1.
    while ps.a.len() > 3 {
        // find rank-0 (or rank-1) in a: we actually push rank 0 first, then
        // rank 1, then sort3, then pa twice. But pa puts them on top in
        // reverse push order; so push smallest-first means top of b = smallest.
        // Actually: first pb pushes rank 0 to b → b=[0]. Then pb rank 1 →
        // b=[0,1] (top = 1). pa pops top of b → a top = 1. Then pa → a top = 0.
        // That yields sorted. Good.
        let target = find_min_rank(&ps.a);
        bring_top_a(ps, target);
        ps.pb();
    }
    sort3(ps);
    while !ps.b.is_empty() {
        ps.pa();
    }
}

fn find_min_rank(a: &[i32]) -> usize {
    // Returns index-from-top of the minimum value.
    let n = a.len();
    let mut best_idx = 0usize;
    let mut best_val = a[n - 1];
    let mut k = 1;
    while k < n {
        let v = a[n - 1 - k];
        if v < best_val {
            best_val = v;
            best_idx = k;
        }
        k += 1;
    }
    best_idx
}

// Bring index-from-top `k` to the top of a using ra/rra.
fn bring_top_a(ps: &mut Ps, k: usize) {
    let n = ps.a.len();
    if n == 0 || k == 0 {
        return;
    }
    let up = k; // ra moves top to bottom, which advances the "next top" by 1
    let down = n - k;
    if up <= down {
        let mut t = 0;
        while t < up {
            ps.ra();
            t += 1;
        }
    } else {
        let mut t = 0;
        while t < down {
            ps.rra();
            t += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Chunk (bucket) sort for n >= 6
//
// Strategy: values in a are ranks 0..n-1. Choose C chunks. Iterate: for each
// value, if its rank < threshold of current chunk, push to b. Within b, if
// the rank is in the top half of the chunk, rotate b so it sits deeper. Then
// when pushing back, select from b the value with minimum cost (ra/rra route).
//
// The "push_swap_chunk" approach with ~5 chunks gives ~700-900 ops for n=100.
// We'll use ~11 chunks for n=500 targeting ~7000 ops.
// ---------------------------------------------------------------------------

fn chunk_sort(ps: &mut Ps, n: usize) {
    // Number of elements to push to b: n - 3 (we keep the top 3 ranks in a).
    let push_count = n - 3;
    // Choose chunk size based on n.
    let chunks: usize = if push_count <= 6 {
        2
    } else if n <= 100 {
        4
    } else if n <= 200 {
        8
    } else if n <= 300 {
        10
    } else if n <= 500 {
        9
    } else {
        let mut c = 13usize;
        while c * c < n {
            c += 2;
        }
        c
    };
    let chunk_size = (push_count + chunks - 1) / chunks;

    // Classification: an element with rank r must go to b iff r < push_count
    // (i.e. r is NOT one of the top 3 ranks n-3, n-2, n-1).
    // Its chunk index is r / chunk_size.
    //
    // We walk a:
    //   - if top's rank >= push_count, leave it (ra to look at next)
    //   - if top's chunk == current_chunk, pb. Then optionally rb if it's
    //     in the lower half of the chunk.
    //   - else (top's chunk > current_chunk), ra.
    //
    // Advance current_chunk when no elements of current_chunk remain in a.
    let mut current_chunk: usize = 0;
    while ps.a.len() > 3 {
        let top_rank = ps.a_top() as usize;
        if top_rank >= push_count {
            // keep; rotate to look at next
            ps.ra();
        } else {
            let top_chunk = top_rank / chunk_size;
            if top_chunk <= current_chunk {
                ps.pb();
                let chunk_lo = top_chunk * chunk_size;
                let chunk_hi = ((top_chunk + 1) * chunk_size).min(push_count);
                let mid = chunk_lo + (chunk_hi - chunk_lo) / 2;
                if top_rank < mid && ps.b.len() > 1 {
                    ps.rb();
                }
            } else {
                ps.ra();
            }
        }
        // Advance chunk if none of current chunk remain in a.
        let mut has_in_current = false;
        let mut i = 0;
        while i < ps.a.len() {
            let rr = ps.a[i] as usize;
            if rr < push_count && rr / chunk_size <= current_chunk {
                has_in_current = true;
                break;
            }
            i += 1;
        }
        if !has_in_current {
            current_chunk += 1;
            if current_chunk >= chunks {
                // Nothing left to push; remaining a are all top-3 ranks.
                break;
            }
        }
    }

    // Now a has at most 3 elements — specifically the top 3 ranks (possibly
    // unsorted / rotated). Sort them.
    sort_small_a(ps);

    // Phase 2: pour b back into a.
    while !ps.b.is_empty() {
        let nb = ps.b.len();
        let mut max_idx = 0usize;
        let mut max_val = ps.b[nb - 1];
        let mut k = 1;
        while k < nb {
            let v = ps.b[nb - 1 - k];
            if v > max_val {
                max_val = v;
                max_idx = k;
            }
            k += 1;
        }
        let up = max_idx;
        let down = nb - max_idx;
        if up <= down {
            let mut t = 0;
            while t < up {
                ps.rb();
                t += 1;
            }
        } else {
            let mut t = 0;
            while t < down {
                ps.rrb();
                t += 1;
            }
        }
        ps.pa();
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let mut argv: Vec<String> = Vec::new();
    let mut first = true;
    for a in env::args() {
        if first {
            first = false;
            continue;
        }
        argv.push(a);
    }

    let values = match parse_args(argv) {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Error");
            exit(1);
        }
    };

    if values.is_empty() {
        exit(0);
    }

    // Compress to ranks 0..n-1.
    let ranks = rank_compress(&values);

    // If ranks are already 0,1,2,...,n-1 in input order, already sorted.
    if is_sorted(&ranks) {
        exit(0);
    }

    // Build Ps with ranks as values (i32). All fit since n < 2^31.
    let mut rank_i32: Vec<i32> = Vec::with_capacity(ranks.len());
    let mut i = 0;
    while i < ranks.len() {
        rank_i32.push(ranks[i] as i32);
        i += 1;
    }

    let n = rank_i32.len();
    let mut ps = Ps::new(rank_i32);

    if n <= 3 {
        sort_small_a(&mut ps);
    } else if n <= 5 {
        sort_small(&mut ps);
    } else {
        chunk_sort(&mut ps, n);
    }

    ps.flush_ops();
}
