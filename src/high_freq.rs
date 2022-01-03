use super::StreamProcessor;
use std::collections::hash_map::{Entry, RandomState};
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct MisraGries<T, const K: usize> {
    pub counts: HashMap<T, usize>,
}

/// A stream processor which returns the highest frequency item if it had a majority.
pub type Majority<T> = MisraGries<T, 1>;

impl<T: Hash + Eq + Clone, const K: usize> StreamProcessor<T> for MisraGries<T, K> {
    fn new() -> Self {
        assert_ne!(K, 0);
        let counts = Default::default();
        Self { counts }
    }
    fn process(&mut self, v: T) {
        let k = self.counts.len();
        match self.counts.entry(v) {
            Entry::Occupied(mut o) => {
                o.insert(o.get() + 1);
            }
            Entry::Vacant(v) if k < K => {
                v.insert(1);
            }
            _ => {
                self.counts.drain_filter(|_, v| {
                    *v -= 1;
                    *v == 0
                });
            }
        }
    }

    // TODO this is painful even with GATs
    type Result = Vec<T>;
    type Args = ();
    fn query(&self, (): &()) -> Self::Result {
        self.counts.keys().cloned().collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CountMin<T, S = RandomState, const BYTES: usize = 32, const H: usize = 16> {
    marker: PhantomData<T>,
    pub buckets: [(S, [u32; BYTES]); H],
}

impl<T: Hash, S: BuildHasher + Default, const B: usize, const H: usize> StreamProcessor<T>
    for CountMin<T, S, B, H>
{
    fn new() -> Self {
        assert_ne!(H, 0);
        assert_ne!(B, 0);
        let buckets = std::array::from_fn(|_| (Default::default(), [0; B]));
        Self {
            marker: Default::default(),
            buckets,
        }
    }
    fn process(&mut self, t: T) {
        assert_ne!(H, 0);
        assert_ne!(B, 0);
        for (s, bucket) in self.buckets.iter_mut() {
            bucket[s.hash_one(&t) as usize % B] += 1;
        }
    }

    type Result = usize;
    type Args = T;
    fn query(&self, t: &T) -> Self::Result {
        assert_ne!(H, 0);
        assert_ne!(B, 0);
        self.buckets
            .iter()
            .map(|(s, bucket)| bucket[s.hash_one(t) as usize % B] as usize)
            .min()
            .unwrap()
    }
}
