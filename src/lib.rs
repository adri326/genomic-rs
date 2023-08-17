#![doc = include_str!("../README.md")]

pub mod chromosome;
use chromosome::*;

pub mod traverse;
use rand::SeedableRng;
use traverse::*;

pub mod genome;
use genome::*;

/// A module to quickly import the necessary types for using this library
pub mod prelude {
    pub use crate::chromosome::Chromosome;
    pub use crate::genome::Genome;
    pub use crate::traverse::{Crossover, CrossoverMethod, Mutator};
}

/// Mutates all of the chromosomes in `genome` with a rate of `rate`.
///
/// A rate of 1.0 means that the chromosomes will be fully scrambled,
/// while a rate of 0.0 means that they should not change.
#[inline]
pub fn mutate<G: Genome>(individual: &mut G, rate: f64, rng: &mut impl rand::Rng) {
    let mut mutator = Mutator::new(rate, rng);

    individual.mutate(&mut mutator);
}

/// Performs the crossover operation on `genome_left` and `genome_right`.
/// Depending on `method`, different chromosomes between the two individuals will be swapped,
/// mixing their genetical code.
///
/// The two individuals should have the same amount of chromosomes (as defined by [Chromosome::size_hint]).
/// If not, then the crossover operation may yield undesired results.
#[inline]
pub fn crossover<G: Genome>(
    individual_left: &mut G,
    individual_right: &mut G,
    method: CrossoverMethod,
    rng: &mut impl rand::Rng,
) {
    debug_assert_eq!(individual_left.size_hint(), individual_right.size_hint());

    let method = match method {
        CrossoverMethod::Uniform(rate) => CrossoverState::Uniform(rate),
        CrossoverMethod::KPoint(amount) => CrossoverState::KPoint {
            count: 0,
            length: individual_left.size_hint() as u64,
            swapped: 0,
            desired: amount,
        },
    };
    let mut crossover = Crossover::new(rng, method);

    individual_left.crossover(individual_right, &mut crossover);
}

/// Reproduces two parent individuals,
/// creating two children with a crossover of the parent's chromosomes and some mutation.
///
/// The two parent should have the same amount of chromosomes (as defined by [Chromosome::size_hint]).
/// If not, then the crossover operation may yield undesired results.
///
/// Each child is mutated with its own `StdRng`,
/// meaning that dropping one of the children should fully optimize out its mutation
/// (assuming this function call gets inlined).
#[inline(always)]
pub fn reproduce<G: Genome + Clone>(
    parent_left: &G,
    parent_right: &G,
    crossover_method: CrossoverMethod,
    mutation_rate: f64,
    rng: &mut impl rand::Rng,
) -> (G, G) {
    debug_assert_eq!(parent_left.size_hint(), parent_right.size_hint());

    let (mut child_left, mut child_right) = (parent_left.clone(), parent_right.clone());

    crossover(&mut child_left, &mut child_right, crossover_method, rng);

    let mut left_rng = rand::rngs::StdRng::from_rng(&mut *rng)
        .expect("Couldn't seed a new rng from the existing rng");
    let mut right_rng = rand::rngs::StdRng::from_rng(&mut *rng)
        .expect("Couldn't seed a new rng from the existing rng");
    mutate(&mut child_left, mutation_rate, &mut left_rng);
    mutate(&mut child_right, mutation_rate, &mut right_rng);

    (child_left, child_right)
}
