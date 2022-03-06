use std::fmt;
use std::hash::Hash;
use std::ops::{Mul, Sub};

use log::{info, trace};

use crate::{BloomFilter, variant_bloom_filter, VariantBloomFilter};

/// Scalable Bloom Filter(SBF)
///
/// SBF is based on variant bloom filter, consists of one or more filter. When the first filter is full,
/// a brand new filter with more k and smaller target false positive rate would be added, extending
/// capacity of SBF while promising target false positive rate.
pub struct ScalableBloomFilter {
    vbfs: Vec<VariantBloomFilter>,
    // slices length
    k0: u32,
    // slice size
    m: u64,
    // treat number of hash functions as a geometric progression, s is it's common ratio
    s: u8,
    // treat vbfs' false positive rate as a geometric progression, r is it's common ratio
    r: f64,
    // target false positive rate
    f: f64,
}

impl ScalableBloomFilter {
    /// Create an empty scalable bloom filter from scratch.
    ///
    /// Generally, user should not use this initializer directly.
    /// Promise the limitation on yourself:
    /// * 0 < k <= u32::MAX
    /// * 0 < m <= u32::MAX
    /// * s = 2 or 4
    /// * 0.8 <= r <= 0.9
    /// * 0 < f < 1
    pub fn from_scratch(first_slices_length: u32, slice_size: u64, s: u8, r: f64, target_false_positive_rate: f64) -> ScalableBloomFilter {
        trace!(target: "ScalableBloomFilter", "from_scratch(k0 = {}, m = {}, s = {}, r = {}, f = {}) called",
            first_slices_length, slice_size, s, r, target_false_positive_rate);
        let vbfs = vec![
            VariantBloomFilter::from_scratch(first_slices_length, slice_size, target_false_positive_rate)
        ];
        ScalableBloomFilter {
            vbfs,
            k0: first_slices_length,
            m: slice_size,
            s,
            r,
            f: target_false_positive_rate,
        }
    }

    /// Create an empty bloom filter with max element's size for the first filter and false positive rate.
    /// The crate would calculate the best buckets length and bucket size, and make s = 4, r = 0.9.
    pub fn new(first_max_size: u64, target_false_positive: f64) -> impl BloomFilter {
        trace!(target: "ScalableBloomFilter", "new(n0 = {}, f = {}) called", first_max_size, target_false_positive);
        assert_ne!(first_max_size, 0_u64);
        assert!(target_false_positive.lt(&1_f64) && target_false_positive.gt(&0_f64));

        let k0 = variant_bloom_filter::utils::calculate_best_k(target_false_positive);
        info!(target: "VariantBloomFilter", "the best k is {}", k0);
        let m = variant_bloom_filter::utils::calculate_best_m(first_max_size, k0, target_false_positive);
        info!(target: "VariantBloomFilter", "the best m is {}", m);
        ScalableBloomFilter::from_scratch(k0, m, 4, 0.9, target_false_positive)
    }

    fn extend(&mut self) {
        let i = self.vbfs.len();
        let ki = (self.s as u32).pow(i as u32).mul(self.k0);
        let fi = (self.r).powf(i as f64).mul(self.f);

        self.vbfs.push(VariantBloomFilter::from_scratch(ki, self.m, fi));
    }
}

impl BloomFilter for ScalableBloomFilter {
    fn add<T>(&mut self, value: &T) -> bool where T: Hash {
        let mut i = self.vbfs.len().sub(1);
        if self.vbfs.get(i).unwrap().is_full() {
            self.extend();
            i = i + 1;
        }
        self.vbfs.get_mut(i).unwrap().add(value)
    }

    fn contains<T>(&self, value: &T) -> bool where T: Hash {
        self.vbfs.iter().any(|vbf| vbf.contains(value))
    }

    fn target_false_positive_rate(&self) -> f64 {
        self.f
    }

    fn current_false_positive_rate(&self) -> f64 {
        1_f64.sub(
            self.vbfs.iter()
                .map(|bfs| 1_f64.sub(bfs.current_false_positive_rate()))
                .fold(1_f64, |res, vbf_f| res * vbf_f)
        )
    }

    fn is_empty(&self) -> bool {
        self.vbfs.get(0).unwrap().is_empty()
    }

    /// Because this is scalable bloom filter, it never gets full
    fn is_full(&self) -> bool {
        false
    }

    fn size(&self) -> u64 {
        self.vbfs.iter().map(|bfs| bfs.size()).sum()
    }

    fn len(&self) -> u64 {
        self.vbfs.iter().map(|bfs| bfs.len()).sum()
    }
}

impl fmt::Display for ScalableBloomFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ScalableBloomFilter")
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use log::debug;

    use super::*;

    fn init() {
        env::set_var("RUST_LOG", "trace");
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn simple_int_test() {
        init();
        let mut bf = ScalableBloomFilter::new(100, 0.001_f64);
        (0..5).for_each(|i| {
            bf.add(&i);
            debug!("false positive is {}", bf.current_false_positive_rate());
        });

        assert!(bf.contains(&2));
        assert!(!bf.contains(&5));
    }

    #[test]
    fn multiple_value_test() {
        init();
        let mut bf = ScalableBloomFilter::new(100, 0.001_f64);

        (-25..25).for_each(|i| {
            bf.add(&i);
        });
        bf.add(&'*');
        bf.add(&"this is a string");

        debug!("false positive is {}", bf.current_false_positive_rate());

        assert!(bf.contains(&2));
        assert!(bf.contains(&5));
    }
}
