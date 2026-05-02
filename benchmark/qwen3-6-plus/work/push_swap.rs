use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        return;
    }

    let mut numbers: Vec<i32> = Vec::new();

    for arg in &args {
        if arg.is_empty() {
            eprintln!("Error");
            process::exit(1);
        }
        for token in arg.split_whitespace() {
            if token.is_empty() {
                eprintln!("Error");
                process::exit(1);
            }
            let val = match parse_i32(token) {
                Some(v) => v,
                None => {
                    eprintln!("Error");
                    process::exit(1);
                }
            };
            let mut is_dup = false;
            let mut j = 0;
            while j < numbers.len() {
                if numbers[j] == val {
                    is_dup = true;
                    break;
                }
                j += 1;
            }
            if is_dup {
                eprintln!("Error");
                process::exit(1);
            }
            numbers.push(val);
        }
    }

    if numbers.is_empty() {
        return;
    }

    if is_sorted(&numbers) {
        return;
    }

    let mut stack_a = numbers;
    let mut stack_b: Vec<i32> = Vec::new();

    let len = stack_a.len();

    if len == 2 {
        if stack_a[0] > stack_a[1] {
            println!("sa");
        }
        return;
    }

    if len == 3 {
        solve3(&mut stack_a);
        return;
    }

    if len == 4 {
        solve4(&mut stack_a, &mut stack_b);
        return;
    }

    if len == 5 {
        solve5(&mut stack_a, &mut stack_b);
        return;
    }

    solve_large(&mut stack_a, &mut stack_b, len);
}

fn parse_i32(s: &str) -> Option<i32> {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return None;
    }

    let mut i = 0;
    let mut negative = false;

    if bytes[0] == b'+' {
        i = 1;
        if i >= bytes.len() {
            return None;
        }
    } else if bytes[0] == b'-' {
        negative = true;
        i = 1;
        if i >= bytes.len() {
            return None;
        }
    }

    let mut result: i32 = 0;

    while i < bytes.len() {
        let digit = bytes[i];
        if digit < b'0' || digit > b'9' {
            return None;
        }
        let d = (digit - b'0') as i32;

        if negative {
            if result < i32::MIN / 10 || (result == i32::MIN / 10 && d > 8) {
                return None;
            }
            result = result * 10 - d;
        } else {
            if result > i32::MAX / 10 || (result == i32::MAX / 10 && d > 7) {
                return None;
            }
            result = result * 10 + d;
        }

        i += 1;
    }

    Some(result)
}

fn is_sorted(a: &[i32]) -> bool {
    let mut i = 0;
    while i + 1 < a.len() {
        if a[i] > a[i + 1] {
            return false;
        }
        i += 1;
    }
    true
}

fn solve3(a: &mut Vec<i32>) {
    let x = a[0];
    let y = a[1];
    let z = a[2];

    if x < y && y < z {
        return;
    } else if x > y && y < z && x > z {
        println!("ra");
    } else if x < y && y > z && x < z {
        println!("sa");
        println!("ra");
    } else if x > y && y < z && x < z {
        println!("sa");
    } else if x > y && y > z {
        println!("sa");
        println!("rra");
    } else {
        println!("rra");
    }
}

fn solve4(a: &mut Vec<i32>, b: &mut Vec<i32>) {
    let min_pos = index_of_min(a);
    rotate_to_top(a, min_pos);
    pb(a, b);
    solve3(a);
    pa(b, a);
    let min_pos = index_of_min(a);
    rotate_to_top(a, min_pos);
}

fn solve5(a: &mut Vec<i32>, b: &mut Vec<i32>) {
    let min1_pos = index_of_min(a);
    rotate_to_top(a, min1_pos);
    pb(a, b);
    let min2_pos = index_of_min(a);
    rotate_to_top(a, min2_pos);
    pb(a, b);
    solve3(a);
    pa(b, a);
    pa(b, a);
}

fn rotate_to_top(a: &mut Vec<i32>, pos: usize) {
    if pos <= a.len() / 2 {
        let mut j = 0;
        while j < pos {
            ra(a);
            j += 1;
        }
    } else {
        let mut j = 0;
        while j < a.len() - pos {
            rra(a);
            j += 1;
        }
    }
}

fn solve_large(a: &mut Vec<i32>, b: &mut Vec<i32>, n: usize) {
    let mut sorted: Vec<i32> = Vec::with_capacity(n);
    let mut i = 0;
    while i < n {
        sorted.push(a[i]);
        i += 1;
    }
    insertion_sort(&mut sorted);

    let n_chunks = if n <= 100 { 8 } else { 15 };

    let mut chunk = 0;
    while chunk < n_chunks {
        let start = chunk * n / n_chunks;
        let end = (chunk + 1) * n / n_chunks;
        let low = sorted[start];
        let high = sorted[end - 1];

        loop {
            let pos = find_in_range(a, low, high);
            if pos >= a.len() {
                break;
            }
            rotate_to_top(a, pos);
            pb(a, b);
        }

        chunk += 1;
    }

    let mut chunk = n_chunks;
    while chunk > 0 {
        chunk -= 1;
        let start = chunk * n / n_chunks;
        let end = (chunk + 1) * n / n_chunks;
        let low = sorted[start];
        let high = sorted[end - 1];

        loop {
            let pos = find_max_in_range(b, low, high);
            if pos >= b.len() {
                break;
            }
            rotate_to_top_b(b, pos);
            pa(b, a);
        }
    }

    let min_pos = index_of_min(a);
    rotate_to_top(a, min_pos);
}

fn find_in_range(a: &[i32], low: i32, high: i32) -> usize {
    let mut best_pos = a.len();
    let mut best_cost = a.len();

    let mut i = 0;
    while i < a.len() {
        if a[i] >= low && a[i] <= high {
            let cost_ra = i;
            let cost_rra = a.len() - i;
            let cost = if cost_ra <= cost_rra { cost_ra } else { cost_rra };
            if cost < best_cost {
                best_cost = cost;
                best_pos = i;
            }
        }
        i += 1;
    }

    best_pos
}

fn find_max_in_range(a: &[i32], low: i32, high: i32) -> usize {
    let mut best_pos = a.len();
    let mut max_val = 0;
    let mut best_cost = a.len();
    let mut found = false;

    let mut i = 0;
    while i < a.len() {
        if a[i] >= low && a[i] <= high {
            let cost_rb = i;
            let cost_rrb = a.len() - i;
            let cost = if cost_rb <= cost_rrb { cost_rb } else { cost_rrb };
            if !found || a[i] > max_val || (a[i] == max_val && cost < best_cost) {
                max_val = a[i];
                best_cost = cost;
                best_pos = i;
                found = true;
            }
        }
        i += 1;
    }

    best_pos
}

fn rotate_to_top_b(a: &mut Vec<i32>, pos: usize) {
    if pos <= a.len() / 2 {
        let mut j = 0;
        while j < pos {
            rb(a);
            j += 1;
        }
    } else {
        let mut j = 0;
        while j < a.len() - pos {
            rrb(a);
            j += 1;
        }
    }
}

fn index_of_min(a: &[i32]) -> usize {
    let mut min_pos = 0;
    let mut i = 1;
    while i < a.len() {
        if a[i] < a[min_pos] {
            min_pos = i;
        }
        i += 1;
    }
    min_pos
}

fn insertion_sort(a: &mut Vec<i32>) {
    let mut i = 1;
    while i < a.len() {
        let key = a[i];
        let mut j = i;
        while j > 0 && a[j - 1] > key {
            a[j] = a[j - 1];
            j -= 1;
        }
        a[j] = key;
        i += 1;
    }
}

fn ra(a: &mut Vec<i32>) {
    if a.len() >= 2 {
        let first = a.remove(0);
        a.push(first);
        println!("ra");
    }
}

fn rra(a: &mut Vec<i32>) {
    if a.len() >= 2 {
        let last = a.pop();
        if let Some(v) = last {
            a.insert(0, v);
        }
        println!("rra");
    }
}

fn rb(b: &mut Vec<i32>) {
    if b.len() >= 2 {
        let first = b.remove(0);
        b.push(first);
        println!("rb");
    }
}

fn rrb(b: &mut Vec<i32>) {
    if b.len() >= 2 {
        let last = b.pop();
        if let Some(v) = last {
            b.insert(0, v);
        }
        println!("rrb");
    }
}

fn pb(a: &mut Vec<i32>, b: &mut Vec<i32>) {
    if !a.is_empty() {
        let val = a.remove(0);
        b.insert(0, val);
        println!("pb");
    }
}

fn pa(b: &mut Vec<i32>, a: &mut Vec<i32>) {
    if !b.is_empty() {
        let val = b.remove(0);
        a.insert(0, val);
        println!("pa");
    }
}
