#![feature(
    int_log,
    generic_const_exprs,
    generic_arg_infer,
    int_abs_diff,
    adt_const_params,
    array_from_fn,
    associated_type_defaults,
    generic_associated_types,
    hash_drain_filter,
    build_hasher_simple_hash_one
)]
#![allow(incomplete_features)]

pub mod adapters;

pub mod bloom;
pub mod count;
pub mod distinct;
pub mod high_freq;
pub mod quantile;
//pub mod compactor;

mod rand;
pub use rand::rand;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

/// Hash function is some function which maps from a set to an index.
pub trait HashFunction<T> {
    const CAP: usize;
    fn hash(&mut self, v: T) -> usize;
}

/// StreamProcessor abstracts over something that processes a stream.
pub trait StreamProcessor<T> {
    /// Create a new empty instance of this processor.
    fn new() -> Self;
    fn process(&mut self, v: T);
    type Result;
    type Args = ();
    fn query(&self, args: &Self::Args) -> Self::Result;

    fn apply(iter: impl Iterator<Item = T>, args: &Self::Args) -> Self::Result
    where
        Self: Sized,
    {
        let mut s = Self::new();
        for elem in iter {
            s.process(elem);
        }
        s.query(args)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct Bitmap<const BYTES: usize> {
    pub bytes: [u8; BYTES],
}

impl<const B: usize> Bitmap<B> {
    const BITS: usize = B * 8;
    pub fn new() -> Self {
        let bytes = [0; B];
        Self { bytes }
    }
    pub fn set(&mut self, i: usize) {
        assert!(i < B * 8);
        let bucket = i / 8;
        let idx = i % 8;
        self.bytes[bucket] |= 1 << idx;
    }
    pub fn set_or_max(&mut self, i: usize) {
        let bucket = i / 8;
        if bucket < B {
            let idx = i % 8;
            self.bytes[bucket] |= 1 << idx;
        } else {
            self.bytes[B - 1] |= 1 << 7;
        }
    }
    pub fn get(&self, i: usize) -> bool {
        assert!(i < B * 8);
        let bucket = i / 8;
        let idx = i % 8;
        1 & (self.bytes[bucket] >> idx) == 1
    }
}
