use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};

use roaring::RoaringBitmap;

use crate::utils::*;

pub struct BloomFilter {
    slices: Vec<RoaringBitmap>,
    // slices_length
    k: u32,
    // slice_size
    m: u32,
    // max_size
    n: u64,
    // target_false_positive
    f: f64,
}

impl BloomFilter {
    // Create an empty bloom filter from scratch.
    //
    // Generally, user should not use this initializer directly.
    // Promise the limitation on yourself:
    // * 0 < k <= u32::MAX
    // * 0 < m <= u32::MAX
    // * 0 < n <= u64::MAX
    // * 0 < f < 1
    pub fn from_scratch(slices_length: u32, slice_size: u32, max_size: u64, target_false_positive: f64) -> BloomFilter {
        let slices: Vec<RoaringBitmap> = (0..slice_size).map(|_| {
            RoaringBitmap::new()
        }).collect();
        BloomFilter {
            slices,
            k: slices_length,
            m: slice_size,
            n: max_size,
            f: target_false_positive,
        }
    }

    // Create an empty bloom filter with max element's size and false positive rate.
    // The crate would calculate the best buckets length and bucket size.
    pub fn new(max_size: u64, target_false_positive: f64) -> BloomFilter {
        assert_ne!(max_size, 0_u64);
        assert!(target_false_positive.lt(&1_f64) && target_false_positive.gt(&0_f64));

        let k = calculate_best_k(target_false_positive);
        let m = calculate_best_m(max_size);
        BloomFilter::from_scratch(k, m, max_size, target_false_positive)
    }

    // Get current false positive rate.
    pub fn false_positive_rate(&self) -> f64 {
        calculate_false_positive_rate(self.m, self.len(), self.k)
    }

    // If this bloom filter is empty.
    pub fn is_empty(&self) -> bool {
        self.slices.iter().all(|slice| {
            slice.is_empty()
        })
    }

    // Get the max size of elements this bloom filter can hold.
    pub fn capacity(&self) -> u64 {
        self.n
    }

    // Get the number of inserted elements
    pub fn len(&self) -> u64 {
        self.slices.iter().map(|slice| slice.len()).sum()
    }
}

impl<T> BloomFilter
    where T: Hash {
    fn get_hash(&self, value: T, seed: u32) -> u64 {
        let mut s = DefaultHasher::new();
        value.hash(&mut s);
        seed.hash(&mut s);
        s.finish()
    }

    pub fn add(&mut self, value: T) {
        todo!()
    }

    pub fn contains(&mut self, value: T) -> bool {
        todo!()
    }
}

impl fmt::Display for BloomFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.rb.len() < 16 {
            write!(f, "RoaringBitmap<{:?}>", self.rb.iter().collect::<Vec<u32>>())
        } else {
            write!(
                f,
                "RoaringBitmap<{:?} values between {:?} and {:?}>",
                self.rb.len(),
                self.rb.min().unwrap(),
                self.rb.max().unwrap()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
