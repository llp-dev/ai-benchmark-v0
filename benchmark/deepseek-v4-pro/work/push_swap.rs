use std::process::exit;
use std::env::args;

fn sa(a: &mut Vec<i32>) {
    if a.len() >= 2 {
        a.swap(0, 1);
        println!("sa");
    }
}

#[allow(dead_code)]
fn sb(b: &mut Vec<i32>) {
    if b.len() >= 2 {
        b.swap(0, 1);
        println!("sb");
    }
}

#[allow(dead_code)]
fn ss(a: &mut Vec<i32>, b: &mut Vec<i32>) {
    let act_a = a.len() >= 2;
    let act_b = b.len() >= 2;
    if act_a {
        a.swap(0, 1);
    }
    if act_b {
        b.swap(0, 1);
    }
    if act_a || act_b {
        println!("ss");
    }
}

fn pa(a: &mut Vec<i32>, b: &mut Vec<i32>) {
    if b.len() > 0 {
        let val = b.remove(0);
        a.insert(0, val);
        println!("pa");
    }
}

fn pb(a: &mut Vec<i32>, b: &mut Vec<i32>) {
    if a.len() > 0 {
        let val = a.remove(0);
        b.insert(0, val);
        println!("pb");
    }
}

fn ra(a: &mut Vec<i32>) {
    if a.len() >= 2 {
        let val = a.remove(0);
        a.push(val);
        println!("ra");
    }
}

#[allow(dead_code)]
fn rb(b: &mut Vec<i32>) {
    if b.len() >= 2 {
        let val = b.remove(0);
        b.push(val);
        println!("rb");
    }
}

#[allow(dead_code)]
fn rr(a: &mut Vec<i32>, b: &mut Vec<i32>) {
    let act_a = a.len() >= 2;
    let act_b = b.len() >= 2;
    if act_a {
        let val = a.remove(0);
        a.push(val);
    }
    if act_b {
        let val = b.remove(0);
        b.push(val);
    }
    if act_a || act_b {
        println!("rr");
    }
}

fn rra(a: &mut Vec<i32>) {
    if a.len() >= 2 {
        let val = a.pop().unwrap();
        a.insert(0, val);
        println!("rra");
    }
}

#[allow(dead_code)]
fn rrb(b: &mut Vec<i32>) {
    if b.len() >= 2 {
        let val = b.pop().unwrap();
        b.insert(0, val);
        println!("rrb");
    }
}

#[allow(dead_code)]
fn rrr(a: &mut Vec<i32>, b: &mut Vec<i32>) {
    let act_a = a.len() >= 2;
    let act_b = b.len() >= 2;
    if act_a {
        let val = a.pop().unwrap();
        a.insert(0, val);
    }
    if act_b {
        let val = b.pop().unwrap();
        b.insert(0, val);
    }
    if act_a || act_b {
        println!("rrr");
    }
}

fn find_min_index(v: &[i32]) -> usize {
    let mut min_i = 0usize;
    let mut i = 1usize;
    while i < v.len() {
        if v[i] < v[min_i] {
            min_i = i;
        }
        i += 1;
    }
    min_i
}

fn rotate_to_top_a(a: &mut Vec<i32>, idx: usize) {
    let len = a.len();
    if idx <= len / 2 {
        let mut k = 0usize;
        while k < idx {
            ra(a);
            k += 1;
        }
    } else {
        let mut k = 0usize;
        let steps = len - idx;
        while k < steps {
            rra(a);
            k += 1;
        }
    }
}

fn push_smallest_to_b(a: &mut Vec<i32>, b: &mut Vec<i32>, count: usize) {
    let mut c = 0usize;
    while c < count {
        let min_idx = find_min_index(a);
        rotate_to_top_a(a, min_idx);
        pb(a, b);
        c += 1;
    }
}

fn sort_three(a: &mut Vec<i32>) {
    let x = a[0];
    let y = a[1];
    let z = a[2];

    if x < y && y < z {
        return;
    }

    if x > y && y < z {
        if x < z {
            sa(a);
        } else {
            ra(a);
        }
    } else if x < y && y > z {
        if x < z {
            rra(a);
            sa(a);
        } else {
            rra(a);
        }
    } else {
        sa(a);
        rra(a);
    }
}

fn sort_four(a: &mut Vec<i32>, b: &mut Vec<i32>) {
    push_smallest_to_b(a, b, 1);
    sort_three(a);
    pa(a, b);
}

fn sort_five(a: &mut Vec<i32>, b: &mut Vec<i32>) {
    push_smallest_to_b(a, b, 2);
    sort_three(a);
    pa(a, b);
    pa(a, b);
}

fn make_indexed(vals: &[i32]) -> Vec<(i32, u32)> {
    let len = vals.len();
    let mut ordered: Vec<i32> = Vec::with_capacity(len);
    {
        let mut i = 0usize;
        while i < len {
            ordered.push(vals[i]);
            i += 1;
        }
    }

    let mut i = 1usize;
    while i < ordered.len() {
        let mut j = i;
        while j > 0 && ordered[j - 1] > ordered[j] {
            ordered.swap(j - 1, j);
            j -= 1;
        }
        i += 1;
    }

    let mut result: Vec<(i32, u32)> = Vec::with_capacity(len);
    {
        let mut vi = 0usize;
        while vi < len {
            let v = vals[vi];
            let mut pos: u32 = 0;
            {
                let mut si = 0usize;
                while si < ordered.len() {
                    if ordered[si] == v {
                        pos = si as u32;
                        break;
                    }
                    si += 1;
                }
            }
            result.push((v, pos));
            vi += 1;
        }
    }
    result
}

fn lookup_idx(indexed: &[(i32, u32)], val: i32) -> u32 {
    let mut i = 0usize;
    while i < indexed.len() {
        if indexed[i].0 == val {
            return indexed[i].1;
        }
        i += 1;
    }
    0
}

fn radix_sort(a: &mut Vec<i32>, b: &mut Vec<i32>) {
    let n = a.len();
    let indexed = make_indexed(a);

    let mut max_bits: u32 = 0;
    {
        let mut t = (n - 1) as u32;
        while t > 0 {
            max_bits += 1;
            t >>= 1;
        }
    }

    let mut bit: u32 = 0;
    while bit < max_bits {
        let mut iter = 0usize;
        while iter < n {
            let val = a[0];
            let idx = lookup_idx(&indexed, val);
            if (idx >> bit) & 1 == 0 {
                pb(a, b);
            } else {
                ra(a);
            }
            iter += 1;
        }
        while b.len() > 0 {
            pa(a, b);
        }
        bit += 1;
    }
}

fn main() {
    let mut a: Vec<i32> = Vec::new();

    let raw: Vec<String> = args().collect();
    if raw.len() < 2 {
        exit(0);
    }

    {
        let mut ai = 1usize;
        while ai < raw.len() {
            let arg = &raw[ai];
            let tokens: Vec<&str> = arg.split_whitespace().collect();
            if tokens.len() == 0 {
                eprintln!("Error");
                exit(1);
            }
            {
                let mut ti = 0usize;
                while ti < tokens.len() {
                    let parsed: Result<i32, _> = tokens[ti].parse();
                    match parsed {
                        Err(_) => {
                            eprintln!("Error");
                            exit(1);
                        }
                        Ok(num) => {
                            {
                                let mut ci = 0usize;
                                while ci < a.len() {
                                    if a[ci] == num {
                                        eprintln!("Error");
                                        exit(1);
                                    }
                                    ci += 1;
                                }
                            }
                            a.push(num);
                        }
                    }
                    ti += 1;
                }
            }
            ai += 1;
        }
    }

    if a.len() == 0 {
        exit(0);
    }

    {
        let mut sorted = true;
        let mut i = 1usize;
        while i < a.len() {
            if a[i - 1] > a[i] {
                sorted = false;
                break;
            }
            i += 1;
        }
        if sorted {
            exit(0);
        }
    }

    let size = a.len();
    let mut b: Vec<i32> = Vec::new();

    if size == 2 {
        if a[0] > a[1] {
            sa(&mut a);
        }
    } else if size == 3 {
        sort_three(&mut a);
    } else if size == 4 {
        sort_four(&mut a, &mut b);
    } else if size == 5 {
        sort_five(&mut a, &mut b);
    } else {
        radix_sort(&mut a, &mut b);
    }
}