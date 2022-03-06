use std::hash::Hash;

/// A scalable bloom filter implementation, based on VariantBloomFilter.
///
/// ```rust
/// use roaring_bloom_filter as bloom_filter;
///
/// let mut bf = bloom_filter::ScalableBloomFilter::new(100, 0.001_f64);
///
/// bf.add(&10);
/// bf.add(&'a');
/// bf.add(&"a string");
///
/// bf.contains(&12);
/// ```
pub use scalable_bloom_filter::ScalableBloomFilter;
/// Stable bloom filter, the best choice for small data set.
/// In this structure, k hash functions share a global bitmap, whose max size is u64::MAX.
/// Usage:
/// ```rust
/// use roaring_bloom_filter as bloom_filter;
///
/// let mut bf = bloom_filter::StableBloomFilter::new(100, 0.001_f64);
///
/// bf.add(&10);
/// bf.add(&'a');
/// bf.add(&"a string");
///
/// bf.contains(&12);
/// ```
pub use stable_bloom_filter::StableBloomFilter;
/// A variant bloom filter, the best choice for bigger data set.
/// In this structure, k hash functions all has it's own slice of bitmap, whose max size is u64::MAX.
/// Usage:
/// ```rust
/// use roaring_bloom_filter as bloom_filter;
///
/// let mut bf = bloom_filter::VariantBloomFilter::new(100, 0.001_f64);
///
/// bf.add(&10);
/// bf.add(&'a');
/// bf.add(&"a string");
///
/// bf.contains(&12);
/// ```
pub use variant_bloom_filter::VariantBloomFilter;

/// Interface for bloom filter.
/// Define all the function a bloom filter should implement.
pub trait BloomFilter {
    /// Add new element into the bloom filter.
    /// Return true when any key are inserted in a slice.
    fn add<T>(&mut self, value: &T) -> bool where T: Hash;

    /// Check if the bloom filter contains the specific key.
    /// Return true when all key are present in all slices, which may contains false positive situation.
    fn contains<T>(&self, value: &T) -> bool where T: Hash;

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

mod stable_bloom_filter;

mod variant_bloom_filter;

mod scalable_bloom_filter;

/// global utils function
pub(crate) mod utils {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    pub fn get_hash<T: Hash>(value: &T, seed: u32) -> u64 {
        let mut s = DefaultHasher::new();
        value.hash(&mut s);
        seed.hash(&mut s);
        s.finish()
    }
}
