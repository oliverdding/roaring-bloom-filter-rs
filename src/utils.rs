use std::intrinsics::{ceilf64, logf64, sqrtf64};
use std::ops::{Div, Mul, Sub};

// See https://rosettacode.org/wiki/Nth_root#Rust
fn nth_root(n: f64, A: f64) -> f64 {
    let p = 1e-9_f64;
    let mut x0 = A / n;

    loop {
        let mut x1 = ((n - 1.0) * x0 + A / f64::powf(x0, n - 1.0)) / n;
        if (x1 - x0).abs() < (x0 * p).abs() { return x1; };
        x0 = x1
    }
}

pub fn calculate_best_m(n: u64) -> u32 {
    1_f64.div(1_f64 - nth_root(n as f64, 0.5_f64)).ceil() as u32
}

pub fn calculate_best_k(f: f64) -> u32 {
    1_f64.div(f).log2().ceil() as u32
}

pub fn calculate_false_positive_rate(m: u32, n: u64, k: u32) -> f64 {
    (1_f64 - (1_f64 - 1_f64.div(m as f64)).powf(n as f64)).powf(k as f64)
}
