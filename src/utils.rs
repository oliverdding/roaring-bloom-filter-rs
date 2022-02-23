use std::ops::{Div, Neg};

pub fn calculate_best_m(n: u64) -> u64 {
    (n as f64).div(0.5_f64.ln()).neg().ceil() as u64
}

pub fn calculate_best_k(f: f64) -> u32 {
    1_f64.div(f).log2().ceil() as u32
}

pub fn calculate_false_positive_rate(m: u64, n: u64, k: u32) -> f64 {
    (1_f64 - (1_f64 - 1_f64.div(m as f64)).powf(n as f64)).powf(k as f64)
}
