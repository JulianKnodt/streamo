use super::{Bitmap, StreamProcessor};
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hash};
use std::marker::PhantomData;

/// Counts the approximate number of distinct elements in an iterator.
/// Uses FM sketch streaming algorithm.
pub struct BloomFilter<T, S = RandomState, const BYTES: usize = 16, const H: usize = 4> {
    marker: PhantomData<T>,
    hashers: [S; H],
    bitmap: Bitmap<BYTES>,
}

impl<T: Hash, S: BuildHasher + Default, const N: usize> StreamProcessor<T>
    for BloomFilter<T, S, N>
{
    fn new() -> Self {
        assert!(N != 0);
        Self {
            marker: Default::default(),
            hashers: Default::default(),
            bitmap: Bitmap::new(),
        }
    }
    fn process(&mut self, v: T) {
        for h in &self.hashers {
            self.bitmap.set(h.hash_one(&v) as usize % Bitmap::<N>::BITS);
        }
    }

    type Result = bool;
    type Args = T;
    fn query(&self, t: &T) -> bool {
        self.hashers
            .iter()
            .all(|h| self.bitmap.get(h.hash_one(&t) as usize % Bitmap::<N>::BITS))
    }
}
