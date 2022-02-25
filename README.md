# bloom-filter

Scalable bloom filter implementation in rust with roaring-bitmap.

> This crate is built on top of the variant of Bloom filters from:
> 
> *F. Chang, W. chang Feng, K. Li, Approximate caches for packet classification.*

## Installation

```toml
[dependencies]
roaring-bloom-filter = "*"
```

## Usage

```rust
extern crate roaring_bloom_filter as bloom_filter;

// Create bloom filter
let mut bf = bloom_filter::BloomFilter::new(100, 0.001_f64);

// You can add any value that implement std::hash::Hash
bf.add(&10);
bf.add(&'a');
bf.add(&"a string");

// Check if exist
bf.contains(&12)

```

## TODO

* [x] Base bloom filter
* [ ] Scalable bloom filter
