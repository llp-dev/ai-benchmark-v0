fn parse_args(args: Vec<String>) -> Result<Vec<i32>, ()> {
    let mut nums: Vec<i32> = Vec::new();
    for arg in args.iter().skip(1) {
        if arg.is_empty() {
            return Err(());
        }
        for tok in arg.split_whitespace() {
            if tok.is_empty() {
                return Err(());
            }
            let negative = tok.as_bytes()[0] == b'-';
            let positive = tok.as_bytes()[0] == b'+';
            let digits = if negative || positive { &tok[1..] } else { tok };
            if digits.is_empty() {
                return Err(());
            }
            for c in digits.chars() {
                if !c.is_ascii_digit() {
                    return Err(());
                }
            }
            if digits.len() > 10 {
                return Err(());
            }
            let val: i64 = if negative {
                let mut v: i64 = 0;
                for c in digits.chars() {
                    v = v * 10 + (c as i64 - '0' as i64);
                }
                -v
            } else {
                let mut v: i64 = 0;
                for c in digits.chars() {
                    v = v * 10 + (c as i64 - '0' as i64);
                }
                v
            };
            if val < i32::MIN as i64 || val > i32::MAX as i64 {
                return Err(());
            }
            let n = val as i32;
            for existing in nums.iter() {
                if *existing == n {
                    return Err(());
                }
            }
            nums.push(n);
        }
    }
    Ok(nums)
}

fn is_sorted(a: &[i32]) -> bool {
    let mut i = 0usize;
    while i + 1 < a.len() {
        if a[i] > a[i + 1] {
            return false;
        }
        i += 1;
    }
    true
}

fn find_min_idx(a: &[i32]) -> usize {
    let mut mi = 0usize;
    let mut i = 1usize;
    while i < a.len() {
        if a[i] < a[mi] {
            mi = i;
        }
        i += 1;
    }
    mi
}

fn find_max_idx(a: &[i32]) -> usize {
    let mut mi = 0usize;
    let mut i = 1usize;
    while i < a.len() {
        if a[i] > a[mi] {
            mi = i;
        }
        i += 1;
    }
    mi
}

fn rotate_up(v: &mut Vec<i32>) {
    if v.len() < 2 {
        return;
    }
    let first = v.remove(0);
    v.push(first);
}

fn rotate_down(v: &mut Vec<i32>) {
    if v.len() < 2 {
        return;
    }
    let last = v.pop().unwrap();
    v.insert(0, last);
}

fn sort_three(a: &mut Vec<i32>) {
    if a.len() <= 1 || is_sorted(a) {
        return;
    }
    if a.len() == 2 {
        println!("sa");
        a.swap(0, 1);
        return;
    }
    let mx = if a[0] > a[1] && a[0] > a[2] {
        0
    } else if a[1] > a[0] && a[1] > a[2] {
        1
    } else {
        2
    };
    if mx == 0 {
        println!("ra");
        rotate_up(a);
    } else if mx == 1 {
        println!("rra");
        rotate_down(a);
    }
    if a[0] > a[1] {
        println!("sa");
        a.swap(0, 1);
    }
}

fn sort_five(a: &mut Vec<i32>, b: &mut Vec<i32>) {
    while a.len() > 3 {
        let mi = find_min_idx(a);
        if mi == 0 {
            println!("pb");
            let v = a.remove(0);
            b.insert(0, v);
        } else if mi <= a.len() / 2 {
            let mut cnt = 0usize;
            while cnt < mi {
                println!("ra");
                rotate_up(a);
                cnt += 1;
            }
            println!("pb");
            let v = a.remove(0);
            b.insert(0, v);
        } else {
            let target = a.len() - mi;
            let mut cnt = 0usize;
            while cnt < target {
                println!("rra");
                rotate_down(a);
                cnt += 1;
            }
            println!("pb");
            let v = a.remove(0);
            b.insert(0, v);
        }
    }
    sort_three(a);
    while !b.is_empty() {
        println!("pa");
        let v = b.remove(0);
        a.insert(0, v);
    }
}

fn all_outside(a: &[i32], lo: i32, hi: i32) -> bool {
    let mut i = 0usize;
    while i < a.len() {
        if a[i] >= lo && a[i] <= hi {
            return false;
        }
        i += 1;
    }
    true
}

fn find_nearest_chunk(a: &[i32], lo: i32, hi: i32) -> Option<(usize, bool)> {
    let mut best_cost = 0usize;
    let mut best_is_top = true;
    let mut found = false;
    let mut i = 0usize;
    while i < a.len() {
        if a[i] >= lo && a[i] <= hi {
            let cost_top = i;
            let cost_bot = a.len() - i;
            let (cost, is_top) = if cost_top <= cost_bot {
                (cost_top, true)
            } else {
                (cost_bot, false)
            };
            if !found || cost < best_cost {
                best_cost = cost;
                best_is_top = is_top;
                found = true;
            }
        }
        i += 1;
    }
    if found {
        Some((best_cost, best_is_top))
    } else {
        None
    }
}

fn push_swap(a: &mut Vec<i32>) {
    let n = a.len();
    if n <= 1 || is_sorted(a) {
        return;
    }
    if n <= 3 {
        sort_three(a);
        return;
    }
    if n <= 5 {
        let mut b: Vec<i32> = Vec::new();
        sort_five(a, &mut b);
        return;
    }
    let mut b: Vec<i32> = Vec::new();
    let k = if n <= 20 { 5 } else { 30 };
    let mut mn = a[0];
    let mut mx = a[0];
    let mut i = 1usize;
    while i < a.len() {
        if a[i] < mn {
            mn = a[i];
        }
        if a[i] > mx {
            mx = a[i];
        }
        i += 1;
    }
    let range = (mx - mn) as f64;
    let chunk_size = if k > 0 { range / k as f64 } else { range };
    let mut c = 0usize;
    while c < k {
        let lo = mn as f64 + chunk_size * c as f64;
        let hi = mn as f64 + chunk_size * (c + 1) as f64;
        let lo_i = lo as i32;
        let hi_i = hi as i32;
        loop {
            if a.is_empty() {
                break;
            }
            if all_outside(a, lo_i, hi_i) {
                break;
            }
            if a[0] >= lo_i && a[0] <= hi_i {
                println!("pb");
                let v = a.remove(0);
                b.insert(0, v);
            } else {
                match find_nearest_chunk(a, lo_i, hi_i) {
                    None => break,
                    Some((cost, is_top)) => {
                        if is_top {
                            let mut cnt = 0usize;
                            while cnt < cost {
                                println!("ra");
                                rotate_up(a);
                                cnt += 1;
                            }
                        } else {
                            let mut cnt = 0usize;
                            while cnt < cost {
                                println!("rra");
                                rotate_down(a);
                                cnt += 1;
                            }
                        }
                    }
                }
            }
        }
        c += 1;
    }
    while !b.is_empty() {
        let max_idx = find_max_idx(&b);
        let r = max_idx;
        let rev = b.len() - max_idx;
        if r == 0 {
            println!("pa");
            let v = b.remove(0);
            a.insert(0, v);
        } else if r < rev {
            let mut cnt = 0usize;
            while cnt < r {
                println!("rb");
                rotate_up(&mut b);
                cnt += 1;
            }
            println!("pa");
            let v = b.remove(0);
            a.insert(0, v);
        } else {
            let mut cnt = 0usize;
            while cnt < rev {
                println!("rrb");
                rotate_down(&mut b);
                cnt += 1;
            }
            println!("pa");
            let v = b.remove(0);
            a.insert(0, v);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let nums = match parse_args(args) {
        Ok(v) => v,
        Err(()) => {
            eprintln!("Error");
            std::process::exit(1);
        }
    };
    if nums.is_empty() || is_sorted(&nums) {
        return;
    }
    let mut a = nums;
    push_swap(&mut a);
}
