#[cfg(test)]
use plotters::prelude::*;

/// Returns a random number given
pub fn rand() -> f32 {
    static mut SEED: f32 = 1.0;
    unsafe {
        SEED += 1.0;
        ((3996.3 * SEED + 42.7).sin() + 1.) / 2.
    }
}

/// A compactor which takes in a stream of elements and outputs
/// every other element, of either odd or even parity.
#[derive(Clone, Debug)]
pub struct Compactor<T> {
    buffer: Vec<T>,
    len: usize,
}

/// Returns number of items in this compactor that are less than the item.
pub fn rank<T: Ord>(outputs: &[T], v: &T) -> usize {
    match outputs.binary_search(v) {
        Ok(i) => i,
        Err(i) => i,
    }
}

impl<T: Ord> Compactor<T> {
    /// Creates a new compactor of a given size.
    pub fn new(max_len: usize) -> Self {
        assert_ne!(max_len, 0, "Cannot pass empty max len to compactor");
        Compactor {
            buffer: Vec::with_capacity(max_len),
            len: max_len,
        }
    }
    /// Adds an item to this compactor, returns true if the compactor has reached capacity
    /// And needs to be compacted.
    pub fn add(&mut self, t: T) -> bool {
        self.buffer.push(t);
        self.buffer.len() == self.len
    }
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn additive_compact(&mut self) -> impl Iterator<Item = T> + '_ {
        assert_eq!(self.len % 2, 0, "Must have even max buffer len");
        self.buffer.sort_unstable();
        let parity = rand().round() as usize;
        self.buffer
            .drain(..)
            .enumerate()
            .filter(move |(i, _)| i % 2 == parity)
            .map(|e| e.1)
    }
    /// Computes an experimental relative error buffer
    pub fn relative_compact(&mut self, buf: &mut Vec<T>) {
        assert!(
            (self.len + 1).is_power_of_two(),
            "Must have buffer len 1 less than pow of 2"
        );
        for i in (0..(self.len + 1).checked_log2().unwrap() as usize) {
            let slice_size = 1 << i;
            let retained = (rand() * slice_size as f32).round() as usize;
            // TODO maybe make this range from something like 1 to 1/8 linearly,
            // instead of decreasing by a geometric sequence.
            for (i, v) in self.buffer.drain(..slice_size).enumerate() {
                if i == retained {
                    buf.push(v)
                }
            }
        }
    }
    /// Linearly compacts with respect to relative error.
    pub fn linear_relative_compact(&mut self, buf: &mut Vec<T>) {
        assert!(self.buffer.len() <= self.len);
        self.buffer.sort_unstable();
        let len = self.len as f32;

        let len_sqrt = len.sqrt();
        let num_chunks = (len_sqrt) as usize;
        let chunk_size = self.len / num_chunks;

        let mut rm_buf = vec![];

        for n in 0..num_chunks {
            if self.buffer.is_empty() {
                break;
            }
            rm_buf.clear();

            let num_to_remove = (n as f32).ceil().min(chunk_size as f32) as usize;
            if num_to_remove >= chunk_size {
                rm_buf.extend(0..chunk_size)
            } else {
                let parity = ((chunk_size as f32) / (num_to_remove as f32)).floor() as usize;
                let mut curr = (rand() * (chunk_size as f32)).floor() as usize % chunk_size;
                while rm_buf.len() < num_to_remove {
                    assert!(!rm_buf.contains(&curr));
                    rm_buf.push(curr);
                    curr = (curr + parity) % chunk_size;
                }
            }
            for (i, v) in self
                .buffer
                .drain(..chunk_size.min(self.buffer.len()))
                .enumerate()
            {
                if !rm_buf.contains(&i) {
                    buf.push(v);
                }
            }
        }
    }
}

pub const fn checked_log(val: usize) -> usize {
    match val.next_power_of_two().checked_log2() {
        Some(v) => v as usize,
        None => unreachable!(),
    }
}

/// Creates a series of N sequential compactors, where each compactor feeds into the next
pub struct ChainedCompactors<T, const N: usize>
where
    [(); N]:,
{
    compactors: [Compactor<T>; N],
}

impl<T: Ord, const N: usize> ChainedCompactors<T, N>
where
    [(); N]:,
{
    pub fn new(compactors: [Compactor<T>; N]) -> Self {
        Self { compactors }
    }
    fn is_empty(&self) -> bool {
        self.compactors.iter().all(|c| c.is_empty())
    }
    fn len(&self) -> usize {
        self.compactors.iter().map(|c| c.len()).sum()
    }
    /// Adds an item to the first compactor in the chain, returning whether it needs to be
    /// compacted.
    pub fn add(&mut self, t: T) -> bool {
        self.compactors[0].add(t)
    }
    pub fn linear_relative_compact(&mut self, into: &mut Vec<T>) {
        self.linear_rel_compact_idx(0, into);
    }
    pub fn linear_relative_compact_all(&mut self, into: &mut Vec<T>) {
        for i in 0..N {
            self.linear_rel_compact_idx(i, into);
        }
    }
    fn linear_rel_compact_idx(&mut self, idx: usize, into: &mut Vec<T>) {
        let mut buf = vec![];
        self.compactors[idx].linear_relative_compact(&mut buf);
        if idx == N - 1 {
            into.append(&mut buf);
            return;
        }
        for v in buf.drain(..) {
            if self.compactors[idx + 1].add(v) {
                self.linear_rel_compact_idx(idx + 1, into)
            }
        }
    }
}

#[test]
fn test_single_additive_compactor() {
    let mut c = Compactor::new(100);
    let mut outputs = vec![];
    let len = 5000;
    for i in (0..len).rev() {
        if c.add(i) {
            outputs.extend(c.additive_compact());
        }
    }
    outputs.sort_unstable();
    assert_eq!(outputs.len(), len / 2);
    assert!(c.is_empty());
}

/*
#[test]
fn test_single_relative_compactor() {
    let mut c = Compactor::new(2500);
    let mut outputs = vec![];
    let len = 10_000;
    let mut inputs = (0..len).rev().filter(|_| rand() > 0.25).collect::<Vec<_>>();
    for &i in inputs.iter() {
        if c.add(i) {
            c.linear_relative_compact(&mut outputs);
        }
    }
    c.linear_relative_compact(&mut outputs);
    assert!(c.is_empty());
    let (add_error, rel_error) = compute_errors(&mut inputs, &mut outputs, len);
    let max_rel_error = rel_error.iter().max_by(|a, b| a.partial_cmp(b).unwrap());
    let max_add_error = add_error.iter().max_by(|a, b| a.partial_cmp(b).unwrap());
    let space_usage = (outputs.len() as f32) / (inputs.len() as f32);
    //println!("{:?}", add_error);
    panic!("[result]: {:?}, {:?}, {:?}",space_usage, max_rel_error, max_add_error);
}
*/

#[test]
fn test_chained_relative_compactor() {
    let mut c = ChainedCompactors::new([
        Compactor::new(144),
        Compactor::new(144),
        Compactor::new(144),
    ]);
    let mut outputs = vec![];
    let len = 10000;
    let mut inputs = (0..len).rev().filter(|_| rand() > 0.25).collect::<Vec<_>>();
    for &i in inputs.iter() {
        if c.add(i) {
            c.linear_relative_compact(&mut outputs);
        }
    }
    c.linear_relative_compact_all(&mut outputs);
    assert!(c.is_empty());
    let (add_error, rel_error) = compute_errors(&mut inputs, &mut outputs, len);
    let max_rel_error = rel_error.iter().max_by(|a, b| a.partial_cmp(b).unwrap());
    let space_usage = (outputs.len() as f32) / (inputs.len() as f32);
    panic!(
        "{:?} \n\n[result]: {:?}, {:?}", rel_error, space_usage, max_rel_error,
    );
}

fn comp_rel_rank(output_rank: usize, output_len: usize) -> f32 {
  let g = output_rank as f32;
  g * 2.05f32.powf((g/output_len as f32))
}

fn compute_errors(inputs: &mut [usize], outputs: &mut [usize], len: usize) -> (Vec<usize>, Vec<f32>) {
    let mut add_error = vec![];
    let mut rel_error = vec![];

    outputs.sort_unstable();
    inputs.sort_unstable();

    let l = len as f32;
    for i in 0usize..len {
        let got = rank(&outputs, &i);
        let got = comp_rel_rank(got, outputs.len()) as usize;
        let exp = rank(&inputs, &i);
        add_error.push(exp.abs_diff(got));

        let exp = exp as f32;
        let got = got as f32;
        if exp == 0.0 && got == 0.0 {
            rel_error.push(0.);
        } else if exp == 0. || got == 0. {
            rel_error.push(1.);
        } else {
            rel_error.push(1. - (exp / got).min(got / exp));
        }
    }

    (add_error, rel_error)
}
