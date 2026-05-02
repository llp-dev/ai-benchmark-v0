use std::env;
use std::process;

fn parse_args() -> Vec<i32> {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut tokens: Vec<String> = Vec::new();

    for arg in &args {
        if arg.is_empty() {
            eprintln!("Error");
            process::exit(1);
        }
        for token in arg.split_whitespace() {
            tokens.push(token.to_string());
        }
    }

    let mut result: Vec<i32> = Vec::new();
    for token in &tokens {
        match token.parse::<i32>() {
            Ok(n) => result.push(n),
            Err(_) => {
                eprintln!("Error");
                process::exit(1);
            }
        }
    }

    // check duplicates
    for i in 0..result.len() {
        for j in (i + 1)..result.len() {
            if result[i] == result[j] {
                eprintln!("Error");
                process::exit(1);
            }
        }
    }

    result
}

fn is_sorted(v: &[i32]) -> bool {
    for i in 1..v.len() {
        if v[i] < v[i - 1] {
            return false;
        }
    }
    true
}

fn do_sa(a: &mut Vec<i32>) {
    if a.len() >= 2 {
        a.swap(0, 1);
    }
    println!("sa");
}

fn do_pa(a: &mut Vec<i32>, b: &mut Vec<i32>) {
    if !b.is_empty() {
        let val = b.remove(0);
        a.insert(0, val);
    }
    println!("pa");
}

fn do_pb(a: &mut Vec<i32>, b: &mut Vec<i32>) {
    if !a.is_empty() {
        let val = a.remove(0);
        b.insert(0, val);
    }
    println!("pb");
}

fn do_ra(a: &mut Vec<i32>) {
    if a.len() >= 2 {
        let val = a.remove(0);
        a.push(val);
    }
    println!("ra");
}

fn do_rb(b: &mut Vec<i32>) {
    if b.len() >= 2 {
        let val = b.remove(0);
        b.push(val);
    }
    println!("rb");
}

fn do_rra(a: &mut Vec<i32>) {
    if a.len() >= 2 {
        let val = a.pop().unwrap();
        a.insert(0, val);
    }
    println!("rra");
}

fn do_rrb(b: &mut Vec<i32>) {
    if b.len() >= 2 {
        let val = b.pop().unwrap();
        b.insert(0, val);
    }
    println!("rrb");
}

// Sort 3 elements in stack a using at most 3 ops
fn sort_3(a: &mut Vec<i32>) {
    loop {
        if a.len() < 2 || is_sorted(a) {
            break;
        }
        if a.len() == 2 {
            if a[0] > a[1] {
                do_sa(a);
            }
            break;
        }
        // 3 elements: top=a[0], mid=a[1], bot=a[2]
        let (top, mid, bot) = (a[0], a[1], a[2]);
        if top < mid && mid < bot {
            break;
        } else if top > mid && mid < bot && top < bot {
            // e.g. 2 1 3 -> sa
            do_sa(a);
        } else if top < mid && mid > bot && top < bot {
            // e.g. 1 3 2 -> ra sa
            do_ra(a);
            do_sa(a);
        } else if top > mid && mid > bot {
            // e.g. 3 2 1 -> sa rra
            do_sa(a);
            do_rra(a);
        } else if top > mid && mid < bot && top > bot {
            // e.g. 3 1 2 -> ra
            do_ra(a);
        } else if top < mid && mid > bot && top > bot {
            // e.g. 2 3 1 -> rra
            do_rra(a);
        } else {
            break;
        }
    }
}

// Compress values to ranks: rank[i] = sorted position of original[i] (0 = smallest)
fn compress_to_ranks(vals: &[i32]) -> Vec<i32> {
    let n = vals.len();
    let mut sorted: Vec<i32> = vals.to_vec();
    // insertion sort
    for i in 1..n {
        let mut j = i;
        while j > 0 && sorted[j] < sorted[j - 1] {
            sorted.swap(j, j - 1);
            j -= 1;
        }
    }
    let mut ranks: Vec<i32> = Vec::new();
    for &v in vals {
        let mut lo = 0usize;
        let mut hi = n;
        while lo < hi {
            let mid = (lo + hi) / 2;
            if sorted[mid] < v {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        ranks.push(lo as i32);
    }
    ranks
}

// Sort 5 elements: push 2 smallest to b, sort 3, insert back. ≤12 ops.
fn sort_5(a: &mut Vec<i32>, b: &mut Vec<i32>) {
    // Push rank 0 (smallest) to b
    let pos0 = a.iter().position(|&v| v == 0).unwrap_or(0);
    let n = a.len();
    if pos0 <= n / 2 {
        for _ in 0..pos0 { do_ra(a); }
    } else {
        for _ in 0..(n - pos0) { do_rra(a); }
    }
    do_pb(a, b);

    // Push rank 1 to b
    let pos1 = a.iter().position(|&v| v == 1).unwrap_or(0);
    let n2 = a.len();
    if pos1 <= n2 / 2 {
        for _ in 0..pos1 { do_ra(a); }
    } else {
        for _ in 0..(n2 - pos1) { do_rra(a); }
    }
    do_pb(a, b);
    // b = [1, 0] (rank 1 on top, rank 0 at bottom)

    sort_3(a);

    // Insert rank 1 (b[0]=1) into sorted a=[2,3,4]
    // rank 1 goes before rank 2 (a[0]=2), so just pa
    do_pa(a, b);
    // a = [1, 2, 3, 4], b = [0]

    // Insert rank 0 (b[0]=0): goes to front, just pa
    do_pa(a, b);
    // a = [0, 1, 2, 3, 4]
}

// Chunk sort for n >= 6
// 1. Push elements from a to b in chunks (by rank), smartly rotating to minimize cost.
//    Larger-ranked elements within a chunk are kept near the top of b.
// 2. Pull from b to a in descending rank order.
fn chunk_sort(a: &mut Vec<i32>, b: &mut Vec<i32>) {
    let n = a.len();

    let chunk_size: usize = if n <= 15 { 4 }
        else if n <= 50 { 9 }
        else if n <= 100 { 20 }
        else if n <= 200 { 35 }
        else { 50 };

    let mut chunk_start = 0usize;

    while chunk_start < n {
        let chunk_end = if chunk_start + chunk_size > n { n } else { chunk_start + chunk_size };
        let chunk_mid = (chunk_start + chunk_end) / 2;
        let target = chunk_end - chunk_start;
        let mut pushed = 0;

        while pushed < target {
            // Find element in a that belongs to [chunk_start, chunk_end) with minimum rotation cost
            let alen = a.len();
            let mut best_pos: Option<usize> = None;
            let mut best_cost = usize::MAX;

            for (i, &v) in a.iter().enumerate() {
                let rank = v as usize;
                if rank >= chunk_start && rank < chunk_end {
                    let cost = if i <= alen / 2 { i } else { alen - i };
                    if cost < best_cost {
                        best_cost = cost;
                        best_pos = Some(i);
                    }
                }
            }

            if let Some(pos) = best_pos {
                let alen = a.len();
                if pos <= alen / 2 {
                    for _ in 0..pos {
                        do_ra(a);
                    }
                } else {
                    for _ in 0..(alen - pos) {
                        do_rra(a);
                    }
                }
                let rank = a[0] as usize;
                do_pb(a, b);
                // Keep higher-ranked elements near top of b for cheaper retrieval
                if rank < chunk_mid {
                    do_rb(b);
                }
                pushed += 1;
            } else {
                break;
            }
        }

        chunk_start = chunk_end;
    }

    // Pull from b to a in descending rank order
    while !b.is_empty() {
        let blen = b.len();
        let mut max_rank = b[0] as usize;
        let mut max_pos = 0usize;
        for (i, &v) in b.iter().enumerate() {
            if v as usize > max_rank {
                max_rank = v as usize;
                max_pos = i;
            }
        }

        if max_pos <= blen / 2 {
            for _ in 0..max_pos {
                do_rb(b);
            }
        } else {
            for _ in 0..(blen - max_pos) {
                do_rrb(b);
            }
        }

        do_pa(a, b);
    }
    // After pulling descending: a[0]=n-1, then n-2, ..., 0 pushed last so a[0]=0
    // Each pa inserts at front: first pa puts n-1 at front, second pa puts n-2 at front -> [n-2,n-1]
    // last pa puts 0 at front -> [0,1,...,n-1]: sorted ascending! Great.
}



fn sort(original: Vec<i32>) {
    let n = original.len();

    if n == 0 {
        return;
    }

    let ranks = compress_to_ranks(&original);
    let mut a: Vec<i32> = ranks;
    let mut b: Vec<i32> = Vec::new();

    if is_sorted(&a) {
        return;
    }

    match n {
        2 => {
            if a[0] > a[1] {
                do_sa(&mut a);
            }
        }
        3 => {
            sort_3(&mut a);
        }
        4 => {
            chunk_sort(&mut a, &mut b);
        }
        5 => {
            sort_5(&mut a, &mut b);
        }
        _ => {
            chunk_sort(&mut a, &mut b);
        }
    }
}

fn main() {
    let vals = parse_args();

    if vals.is_empty() {
        return;
    }

    if is_sorted(&vals) {
        return;
    }

    sort(vals);
}
