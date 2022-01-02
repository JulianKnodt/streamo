use super::{rand, StreamProcessor};
use crate::count::ExactCounter;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Quantile<
    T,
    const EXP_SIZE: usize = 4096,
    const SAMPLE_SIZE: usize = 128,
    C = ExactCounter,
> {
    samples: Vec<T>,
    /// Maintains count of how many elements have been seen
    pub counter: C,
}

impl<T, const E: usize, const K: usize, C> Quantile<T, E, K, C> {
    const CHANCE: f32 = (E as f32 / K as f32);
}

impl<T: Ord, const E: usize, const K: usize, C: StreamProcessor<(), Args = (), Result = usize>>
    StreamProcessor<T> for Quantile<T, E, K, C>
{
    fn new() -> Self {
        Self {
            samples: Vec::with_capacity(K),
            counter: C::new(),
        }
    }
    fn process(&mut self, t: T) {
        self.counter.process(());
        if rand() > Self::CHANCE {
            return;
        }
        let idx = self.samples.binary_search(&t);
        if let Err(idx) = idx {
            self.samples.insert(idx, t);
        }
    }
    type Result = usize;
    type Args = T;
    /// Returns the rank of an item.
    fn query(&self, a: &T) -> usize {
        let count = self.counter.query(&()) as f32;
        // In case it was a lie about how large the stream is, keep a count instead of using
        // expected size.
        let i = match self.samples.binary_search(a) {
            Ok(i) | Err(i) => i as f32,
        };
        let sampled = self.samples.len() as f32;
        (count * i / sampled) as usize
    }
}
