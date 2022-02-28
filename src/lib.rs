mod variant_bloom_filter;

use std::hash::Hash;

/// Interface for bloom filter.
/// Define all the function a bloom filter should implement.
pub trait BloomFilter {
    /// Add new element into the bloom filter.
    /// Return true when any key are inserted in a slice.
    fn add<T>(&mut self, value: &T) -> bool where T: Hash;

    /// Check if the bloom filter contains the specific key.
    /// Return true when all key are present in all slices, which may contains false positive situation.
    fn contains<T>(&mut self, value: &T) -> bool where T: Hash;

    /// Get target false positive rate.
    fn target_false_positive_rate(&self) -> f64;

    /// Get current false positive rate.
    fn current_false_positive_rate(&self) -> f64;

    /// If this bloom filter is empty.
    fn is_empty(&self) -> bool;

    /// If this bloom filter is full.
    fn is_full(&self) -> bool;

    /// Get the number of inserted elements in this bloom filter.
    fn size(&self) -> u64;

    /// Get the number of inserted bits in all slices.
    fn len(&self) -> u64;
}

pub use variant_bloom_filter::VariantBloomFilter;
