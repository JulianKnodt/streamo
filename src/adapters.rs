use super::StreamProcessor;
use std::array;

/// Gets a better approximation of an approximation by taking the median of many instances.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MedianOfMeans<S, const N: usize, const M: usize> {
    groups: [[S; N]; M],
}

impl<T: Copy, S: StreamProcessor<T>, const N: usize, const M: usize> StreamProcessor<T>
    for MedianOfMeans<S, N, M>
{
    fn new() -> Self {
        let groups = array::from_fn(|_| array::from_fn(|_| S::new()));
        Self { groups }
    }
    fn process(&mut self, v: T) {
        for group in self.groups.iter_mut() {
            for sub in group.iter_mut() {
                sub.process(v);
            }
        }
    }
    type Result = S::Result;
    type Args = S::Args;
    fn query(&self, _args: &S::Args) -> S::Result {
        // TODO need to make this work with numeric types.
        todo!();
    }
}

/// Get better bounds on boolean operations
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BoolGroup<S, const N: usize> {
    subs: [S; N],
}

impl<T: Copy, S: StreamProcessor<T, Args = T, Result = bool>, const N: usize> StreamProcessor<T>
    for BoolGroup<S, N>
{
    fn new() -> Self {
        let subs = array::from_fn(|_| S::new());
        Self { subs }
    }
    fn process(&mut self, v: T) {
        for sub in self.subs.iter_mut() {
            sub.process(v);
        }
    }
    type Result = bool;
    type Args = T;
    fn query(&self, args: &T) -> S::Result {
        self.subs.iter().all(|sub| sub.query(args))
    }
}
