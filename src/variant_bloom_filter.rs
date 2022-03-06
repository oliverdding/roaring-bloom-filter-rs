use std::fmt;
use std::hash::Hash;
use std::ops::{BitOr, Div};

use log::{debug, info, trace};
use roaring::RoaringTreemap;

use crate::BloomFilter;

/// Variant Bloom Filter(VBF)
///
/// Every hash function has it's own slice instead of sharing the whole bitmap.
/// This introduce the possibility of concurrency of manipulating multiply hash function at the same time.
pub struct VariantBloomFilter {
    slices: Vec<RoaringTreemap>,
    // slices length
    k: u32,
    // slice size
    m: u64,
    // counter for inserted elements
    n: u64,
    // target false positive rate
    f: f64,
}

impl VariantBloomFilter {
    /// Create an empty variant bloom filter from scratch.
    ///
    /// Generally, user should not use this initializer directly.
    /// Promise the limitation on yourself:
    /// * 0 < k <= u32::MAX
    /// * 0 < m <= u32::MAX
    /// * 0 < f < 1
    pub fn from_scratch(slices_length: u32, slice_size: u64, target_false_positive_rate: f64) -> VariantBloomFilter {
        trace!(target: "VariantBloomFilter", "from_scratch(k = {}, m = {}, f = {}) called",
            slices_length, slice_size, target_false_positive_rate);
        let slices: Vec<RoaringTreemap> = (0..slices_length).map(|_| {
            RoaringTreemap::new()
        }).collect();
        VariantBloomFilter {
            slices,
            k: slices_length,
            m: slice_size,
            n: 0,
            f: target_false_positive_rate,
        }
    }

    /// Create an empty bloom filter with max element's size and false positive rate.
    /// The crate would calculate the best buckets length and bucket size.
    pub fn new(max_size: u64, target_false_positive: f64) -> impl BloomFilter {
        trace!(target: "VariantBloomFilter", "new(n = {}, f = {}) called", max_size, target_false_positive);
        assert_ne!(max_size, 0_u64);
        assert!(target_false_positive.lt(&1_f64) && target_false_positive.gt(&0_f64));

        let k = utils::calculate_best_k(target_false_positive);
        info!(target: "VariantBloomFilter", "the best k is {}", k);
        let m = utils::calculate_best_m(max_size, k, target_false_positive);
        info!(target: "VariantBloomFilter", "the best m is {}", m);
        VariantBloomFilter::from_scratch(k, m, target_false_positive)
    }
}

impl BloomFilter for VariantBloomFilter {
    fn add<T>(&mut self, value: &T) -> bool where T: Hash {
        self.n = self.n + 1;
        (0..self.k).map(|i| {
            let key = crate::utils::get_hash(value, i) % self.m;
            debug!(target: "VariantBloomFilter", "inserting the key: {}", key);
            self.slices[i as usize].insert(key)
        }).fold(false, |res, is_exist| res.bitor(is_exist)) // cannot use any() here
    }

    fn contains<T>(&self, value: &T) -> bool where T: Hash {
        (0..self.k).all(|i| {
            let key = crate::utils::get_hash(value, i) % self.m;
            debug!(target: "VariantBloomFilter", "checking the key: {}", key);
            self.slices[i as usize].contains(key)
        })
    }

    fn target_false_positive_rate(&self) -> f64 {
        self.f
    }

    fn current_false_positive_rate(&self) -> f64 {
        self.slices.iter().map(|slice| {
            (slice.len() as f64).div(self.m as f64)
        }).fold(1_f64, |res, slice_f| res * slice_f)
    }

    fn is_empty(&self) -> bool {
        self.slices.iter().all(|slice| {
            slice.is_empty()
        })
    }

    fn is_full(&self) -> bool {
        self.current_false_positive_rate() >= self.target_false_positive_rate()
    }

    fn size(&self) -> u64 {
        self.n
    }

    fn len(&self) -> u64 {
        self.slices.iter().map(|slice| slice.len()).sum()
    }
}

impl fmt::Display for VariantBloomFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VariantBloomFilter")
    }
}

pub(crate) mod utils {
    use std::ops::{Div, Mul};

    pub fn calculate_best_m(n: u64, k: u32, f: f64) -> u64 {
        (n as f64).mul(f.ln().abs()).div((k as f64).mul(2_f64.ln().powi(2))).ceil() as u64
    }

    pub fn calculate_best_k(f: f64) -> u32 {
        1_f64.div(f).log2().ceil() as u32
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    fn init() {
        env::set_var("RUST_LOG", "trace");
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn simple_int_test() {
        init();
        let mut bf = VariantBloomFilter::new(100, 0.001_f64);
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
        let mut bf = VariantBloomFilter::new(100, 0.001_f64);

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
