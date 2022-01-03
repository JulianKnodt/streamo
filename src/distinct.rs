use super::{Bitmap, StreamProcessor};
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hash};
use std::marker::PhantomData;

/// Counts the approximate number of distinct elements in an iterator.
/// Uses FM sketch streaming algorithm.
pub struct FlajoletMartin<T, S = RandomState, const BYTES: usize = 8> {
    marker: PhantomData<T>,
    state: S,
    bitmap: Bitmap<BYTES>,
}

const PHI: f32 = 0.77351;
impl<T: Hash, S: BuildHasher + Default, const N: usize> StreamProcessor<T>
    for FlajoletMartin<T, S, N>
{
    fn new() -> Self {
        assert_ne!(N, 0);
        Self {
            marker: Default::default(),
            state: Default::default(),
            bitmap: Bitmap::new(),
        }
    }
    fn process(&mut self, v: T) {
        assert_ne!(N, 0);
        self.bitmap
            .set_or_max(self.state.hash_one(v).trailing_zeros() as usize)
    }

    type Result = usize;
    fn query(&self, (): &()) -> usize {
        assert_ne!(N, 0);
        let approx = |n: usize| (2f32.powi(n as i32) / PHI) as usize - 1;

        let rev = self.bitmap.bytes.iter().enumerate();
        for (i, &elem) in rev {
            if elem == u8::MAX {
                continue;
            }

            return approx(i * 8 + elem.trailing_ones() as usize);
        }
        approx(N * 8)
    }
}

#[cfg(test)]
mod test_distinct {
    use crate::StreamProcessor;
    use std::collections::hash_map::RandomState;
    #[test]
    fn empty() {
        assert_eq!(
            0,
            super::FlajoletMartin::<&u32, RandomState, 8>::apply(vec![].into_iter(), &())
        )
    }
    // probabilistic
    quickcheck! {
      fn fm(x: Vec<u32>) -> bool {
        let distinct_count = super::FlajoletMartin::<_, RandomState, 8>::apply(x.iter(), &());
        let mut x = x.clone();
        x.sort_unstable();
        x.dedup();
        x.len().abs_diff(distinct_count) < 1000+x.len()/2
      }
    }
}
