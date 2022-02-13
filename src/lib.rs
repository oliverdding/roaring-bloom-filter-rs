use std::fmt;
use std::ops::{Div, Sub};

use roaring::RoaringBitmap;

pub struct BloomFilter<T> {
    rb: RoaringBitmap,
    m: u64,
    fn_vec: Vec<fn(&T) -> u32>,
}

impl<T> BloomFilter<T> {
    pub fn new(m: u64, fn_vec: &[fn(&T) -> u32]) -> BloomFilter<T> {
        assert_ne!(m, 0);
        assert_ne!(fn_vec.len(), 0);
        assert!(fn_vec.len() < m as usize);
        BloomFilter { rb: RoaringBitmap::new(), m, fn_vec: fn_vec.to_vec() }
    }

    pub fn add(&mut self, value: T) {
        self.fn_vec.iter().map(|hash_fn: &fn(&T) -> u32| {
            hash_fn(&value)
        }).for_each(|key: u32| {
            println!("add: {}", key);
            self.rb.insert(key);
        });
    }

    pub fn contains(&mut self, value: T) -> bool {
        self.fn_vec.iter().map(|hash_fn: &fn(&T) -> u32| {
            hash_fn(&value)
        }).all(|key: u32| {
            println!("con: {}", key);
            self.rb.contains(key)
        })
    }

    pub fn false_positive_rate(&self) -> f64 {
        let m = self.m as f64;
        let n = self.rb.len() as f64;
        let k = self.fn_vec.len() as f64;
        (1_f64.sub((1_f64.sub(1_f64.div(m))).powf(k * n))).powf(k)
    }

    pub fn is_empty(&self) -> bool {
        self.rb.is_empty()
    }

    pub fn capacity(&self) -> u64 {
        self.m
    }

    pub fn len(&self) -> u64 {
        self.rb.len()
    }
}

impl<T> fmt::Display for BloomFilter<T> {
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
    use fasthash::*;

    use crate::BloomFilter;

    fn hash(value: &[u8], seed: u32, m: u32) -> u32 {
        murmur3::hash32_with_seed(value, seed) % m
    }

    #[test]
    fn simple_str_test() {
        fn hash1(value: &String) -> u32 {
            hash(value.as_ref(), 0, 10)
        }
        fn hash2(value: &String) -> u32 {
            hash(value.as_ref(), 1, 10)
        }

        let mut bf = BloomFilter::new(10, &[hash1, hash2]);
        bf.add(String::from("hello"));
        println!("{}", bf.false_positive_rate());
        bf.add(String::from("mckas"));
        println!("{}", bf.false_positive_rate());

        println!("{}", bf);

        assert!(bf.contains(String::from("hello")));
        assert!(bf.contains(String::from("charmer"))); // false positive
        assert!(!bf.contains(String::from("world")))
    }

    #[test]
    fn simple_int_test() {
        fn hash1(value: &i32) -> u32 {
            hash(value.to_be_bytes().as_ref(), 0, 10)
        }
        fn hash2(value: &i32) -> u32 {
            hash(value.to_be_bytes().as_ref(), 1, 10)
        }

        let mut bf = BloomFilter::new(10, &[hash1, hash2]);
        bf.add(2022);
        println!("{}", bf.false_positive_rate());
        bf.add(1970);
        println!("{}", bf.false_positive_rate());
        bf.add(0);
        println!("{}", bf.false_positive_rate());

        println!("{}", bf);

        assert!(bf.contains(2022));
        assert!(bf.contains(2000)); // false positive
        assert!(!bf.contains(1997));
    }
}
