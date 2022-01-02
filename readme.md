# Streamo

Streaming algorithms in Rust.

Built after taking a [course on Streaming
Algorithms](https://www.cs.princeton.edu/~hy2/teaching/streaming.html).

## What is a streaming algorithm?

A Streaming Algoirthm is an algorithm that works over some (potentially large) data set and
computes some aggregate evaluation of the items in it. For example, computing number of distinct
elements in a stream, the median, etc. Generally they should operate in low-memory, given that
the dataset may be essentially infinite.

This repo contains a few convenient ones.

All streaming algorithms follow the following trait:
```rust
pub trait StreamProcessor<T> {
    fn new() -> Self;

    fn process(&mut self, v: T);
    type Result;
    type Args = ();
    fn query(&self, args: &Self::Args) -> Self::Result;

    ...
}
```

It is intended to be used in conjunction with standard Rust iterators:
```
let s: StreamProcessor = <...>;
for t in iter {
  s.process(t);
}
let val = s.query(<...>);
// do something with val
```
