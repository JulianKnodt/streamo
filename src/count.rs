use super::{rand, StreamProcessor};

/// Keeps an approximate count of a very large stream
/// Returning the total number of elements within a constant factor.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MorrisCounter<const ALPHA: f32> {
    count: usize,
}

impl<T, const A: f32> StreamProcessor<T> for MorrisCounter<A> {
    fn new() -> Self {
        Self { count: 0 }
    }
    fn process(&mut self, _: T) {
        let r = rand();
        if r < (1.0 + A).powi(self.count as i32).recip() {
            self.count += 1;
        }
    }
    type Result = usize;
    fn query(&self, (): &()) -> usize {
        let approx = ((1.0 + A).powi(self.count as i32) - 1.0) / A;
        approx.round() as usize
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ExactCounter {
    count: usize,
}

impl<T> StreamProcessor<T> for ExactCounter {
    fn new() -> Self {
        Self { count: 0 }
    }
    fn process(&mut self, _: T) {
        self.count += 1;
    }
    type Result = usize;
    fn query(&self, (): &()) -> usize {
        self.count
    }
}
