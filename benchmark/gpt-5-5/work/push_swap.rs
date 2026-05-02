fn bad_input() -> ! {
    eprintln!("Error");
    std::process::exit(1);
}

fn valid_token(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut i = 0usize;

    if bytes.len() == 0 {
        return false;
    }
    if bytes[0] == b'+' || bytes[0] == b'-' {
        if bytes.len() == 1 {
            return false;
        }
        i = 1;
    }
    while i < bytes.len() {
        if bytes[i] < b'0' || bytes[i] > b'9' {
            return false;
        }
        i += 1;
    }
    true
}

fn read_values() -> Vec<i32> {
    let mut values: Vec<i32> = Vec::new();
    let mut first = true;

    for arg in std::env::args() {
        if first {
            first = false;
            continue;
        }
        if arg.len() == 0 {
            bad_input();
        }
        let mut saw = false;
        for token in arg.split_whitespace() {
            saw = true;
            if !valid_token(token) {
                bad_input();
            }
            let value = match token.parse::<i32>() {
                Ok(v) => v,
                Err(_) => bad_input(),
            };
            let mut i = 0usize;
            while i < values.len() {
                if values[i] == value {
                    bad_input();
                }
                i += 1;
            }
            values.push(value);
        }
        if !saw {
            bad_input();
        }
    }
    values
}

fn ranks(values: &Vec<i32>) -> Vec<i32> {
    let mut out: Vec<i32> = Vec::new();
    let mut i = 0usize;

    while i < values.len() {
        let mut rank = 0i32;
        let mut j = 0usize;
        while j < values.len() {
            if values[j] < values[i] {
                rank += 1;
            }
            j += 1;
        }
        out.push(rank);
        i += 1;
    }
    out
}

fn ordered(a: &Vec<i32>) -> bool {
    let mut i = 1usize;

    while i < a.len() {
        if a[i - 1] > a[i] {
            return false;
        }
        i += 1;
    }
    true
}

fn op_sa(a: &mut Vec<i32>) {
    if a.len() > 1 {
        a.swap(0, 1);
    }
    println!("sa");
}

fn op_pa(a: &mut Vec<i32>, b: &mut Vec<i32>) {
    if b.len() > 0 {
        let v = b.remove(0);
        a.insert(0, v);
    }
    println!("pa");
}

fn op_pb(a: &mut Vec<i32>, b: &mut Vec<i32>) {
    if a.len() > 0 {
        let v = a.remove(0);
        b.insert(0, v);
    }
    println!("pb");
}

fn op_ra(a: &mut Vec<i32>) {
    if a.len() > 1 {
        let v = a.remove(0);
        a.push(v);
    }
    println!("ra");
}

fn op_rra(a: &mut Vec<i32>) {
    if a.len() > 1 {
        let last = a.len() - 1;
        let v = a.remove(last);
        a.insert(0, v);
    }
    println!("rra");
}

fn three(a: &mut Vec<i32>) {
    let x = a[0];
    let y = a[1];
    let z = a[2];

    if x < y && y < z {
        return;
    }
    if x > y && y < z && x < z {
        op_sa(a);
    } else if x > y && y > z {
        op_sa(a);
        op_rra(a);
    } else if x > y && y < z && x > z {
        op_ra(a);
    } else if x < y && y > z && x < z {
        op_sa(a);
        op_ra(a);
    } else {
        op_rra(a);
    }
}

fn pos_of(a: &Vec<i32>, target: i32) -> usize {
    let mut i = 0usize;

    while i < a.len() {
        if a[i] == target {
            return i;
        }
        i += 1;
    }
    a.len()
}

fn bring_top(a: &mut Vec<i32>, target: i32) {
    let p = pos_of(a, target);
    let len = a.len();

    if p <= len / 2 {
        let mut i = 0usize;
        while i < p {
            op_ra(a);
            i += 1;
        }
    } else {
        let mut i = p;
        while i < len {
            op_rra(a);
            i += 1;
        }
    }
}

fn small(a: &mut Vec<i32>, b: &mut Vec<i32>) {
    let n = a.len();

    if n == 2 {
        if a[0] > a[1] {
            op_sa(a);
        }
    } else if n == 3 {
        three(a);
    } else {
        let keep = n - 3;
        let mut target = 0i32;
        while target < keep as i32 {
            bring_top(a, target);
            op_pb(a, b);
            target += 1;
        }
        three(a);
        while b.len() > 0 {
            op_pa(a, b);
        }
    }
}

fn big(a: &mut Vec<i32>, b: &mut Vec<i32>) {
    let n = a.len();
    let mut max = n - 1;
    let mut bits = 0usize;

    while max > 0 {
        bits += 1;
        max = max / 2;
    }
    let mut bit = 0usize;
    while bit < bits {
        let mut i = 0usize;
        while i < n {
            if ((a[0] >> bit) & 1) == 0 {
                op_pb(a, b);
            } else {
                op_ra(a);
            }
            i += 1;
        }
        while b.len() > 0 {
            op_pa(a, b);
        }
        bit += 1;
    }
}

fn main() {
    let values = read_values();

    if values.len() == 0 {
        return;
    }
    let mut a = ranks(&values);
    let mut b: Vec<i32> = Vec::new();

    if ordered(&a) {
        return;
    }
    if a.len() <= 5 {
        small(&mut a, &mut b);
    } else {
        big(&mut a, &mut b);
    }
}
