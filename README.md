# genomic.rs

A small crate to help implement genetical algorithms in Rust.

This crate defines two traits:
- `Chromosome`s represent a single trainable parameter.
    The `Chromosome` trait is implemented for a few built-in types, and you can implement it on your own.
- `Genome` represents a set of `Chromosome`s.

Implementing the `Genome` trait is done in a declarative fashion:

```rust
use genomic::prelude::*;
use genomic::mutate;
use genomic::chromosome::UniformCh;

pub struct Triple {
    first: i32,
    second: i8,
    third: f32,
}

impl Genome for Triple {
    fn mutate(&mut self, mutator: &mut Mutator<impl rand::Rng>) {
        mutator
            .chromosome(&mut self.first)
            .chromosome(&mut self.second)
            // For floats, we need to choose a method and bounds for mutating them:
            .wrap_ch(UniformCh::new(self.third, 0.0, 1.0), &mut self.third);
    }

    fn crossover(&mut self, other: &mut Self, crossover: &mut Crossover<impl rand::Rng>) {
        crossover
            .chromosome(&mut self.first, &mut other.first)
            .chromosome(&mut self.second, &mut other.second)
            .chromosome(&mut self.third, &mut other.third);
    }

    fn size_hint(&self) -> usize {
        // We have three chromosomes
        3
    }
}
```
