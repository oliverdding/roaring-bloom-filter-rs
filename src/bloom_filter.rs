use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{BitOr, Div};

use log::{debug, info, trace};

use roaring::RoaringTreemap;

use crate::utils::*;

/// A variant of bloom filter
///
/// Every hash function has it's own slice instead of sharing the whole bitmap.
/// This introduce the possibility of concurrency of manipulating multiply hash function at the same time.
pub struct BloomFilter {
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

impl BloomFilter {
    /// Create an empty bloom filter from scratch.
    ///
    /// Generally, user should not use this initializer directly.
    /// Promise the limitation on yourself:
    /// * 0 < k <= u32::MAX
    /// * 0 < m <= u32::MAX
    /// * 0 < f < 1
    pub fn from_scratch(slices_length: u32, slice_size: u64, target_false_positive_rate: f64) -> BloomFilter {
        trace!(target: "BloomFilter", "from_scratch(k = {}, m = {}, f = {}) called",
            slices_length, slice_size, target_false_positive_rate);
        let slices: Vec<RoaringTreemap> = (0..slices_length).map(|_| {
            RoaringTreemap::new()
        }).collect();
        BloomFilter {
            slices,
            k: slices_length,
            m: slice_size,
            n: 0,
            f: target_false_positive_rate,
        }
    }

    /// Create an empty bloom filter with max element's size and false positive rate.
    /// The crate would calculate the best buckets length and bucket size.
    pub fn new(max_size: u64, target_false_positive: f64) -> BloomFilter {
        trace!(target: "BloomFilter", "new(n = {}, f = {}) called", max_size, target_false_positive);
        assert_ne!(max_size, 0_u64);
        assert!(target_false_positive.lt(&1_f64) && target_false_positive.gt(&0_f64));

        let k = calculate_best_k(target_false_positive);
        info!(target: "BloomFilter", "the best k is {}", k);
        let m = calculate_best_m(max_size);
        info!(target: "BloomFilter", "the best m is {}", m);
        BloomFilter::from_scratch(k, m, target_false_positive)
    }

    /// Add new element into the bloom filter.
    /// Return true when any key are inserted in a slice.
    pub fn add<T>(&mut self, value: &T) -> bool
        where T: Hash {
        trace!(target: "BloomFilter", "add() called");
        self.n = self.n + 1;
        (0..self.k).map(|i| {
            let key = self.get_hash(value, i) % self.m;
            debug!(target: "BloomFilter", "inserting the key: {}", key);
            self.slices[i as usize].insert(key)
        }).fold(false, |res, is_exist| res.bitor(is_exist)) // cannot use any() here
    }

    /// Check if the bloom filter contains the specific key.
    /// Return true when all key are present in all slices, which may contains false positive situation.
    pub fn contains<T>(&mut self, value: &T) -> bool
        where T: Hash {
        trace!(target: "BloomFilter", "contains() called");
        (0..self.k).all(|i| {
            let key = self.get_hash(value, i) % self.m;
            debug!(target: "BloomFilter", "checking the key: {}", key);
            self.slices[i as usize].contains(key)
        })
    }

    fn get_hash<T: Hash>(&self, value: &T, seed: u32) -> u64 {
        let mut s = DefaultHasher::new();
        value.hash(&mut s);
        seed.hash(&mut s);
        s.finish()
    }

    /// Get target false positive rate.
    pub fn target_false_positive_rate(&self) -> f64 {
        trace!(target: "BloomFilter", "target_false_positive_rate() called");
        self.f
    }

    /// Get current false positive rate.
    pub fn current_false_positive_rate(&self) -> f64 {
        trace!(target: "BloomFilter", "current_false_positive_rate() called");
        self.slices.iter().map(|slice| {
            (slice.len() as f64).div(self.m as f64)
        }).fold(1_f64, |res, slice_f| res * slice_f)
    }

    /// If this bloom filter is empty.
    pub fn is_empty(&self) -> bool {
        trace!(target: "BloomFilter", "is_empty() called");
        self.slices.iter().all(|slice| {
            slice.is_empty()
        })
    }

    /// If this bloom filter is full.
    pub fn is_full(&self) -> bool {
        trace!(target: "BloomFilter", "is_full() called");
        self.current_false_positive_rate() >= self.target_false_positive_rate()
    }

    /// Get the number of inserted elements in this bloom filter.
    pub fn size(&self) -> u64 {
        trace!(target: "BloomFilter", "len() called");
        self.n
    }

    /// Get the number of inserted bits in all slices.
    pub fn len(&self) -> u64 {
        trace!(target: "BloomFilter", "len() called");
        self.slices.iter().map(|slice| slice.len()).sum()
    }
}

impl fmt::Display for BloomFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BloomFilter")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::env;

    fn init() {
        env::set_var("RUST_LOG", "trace");
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn simple_int_test() {
        init();
        let mut bf = BloomFilter::new(100, 0.001_f64);
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
        let mut bf = BloomFilter::new(100, 0.001_f64);

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
