#![doc = include_str!("../README.md")]

pub mod chromosome;
use chromosome::*;

pub mod traverse;
use traverse::*;

pub mod genome;
use genome::*;

/// A module to quickly import the necessary types for using this library
pub mod prelude {
    pub use crate::chromosome::Chromosome;
    pub use crate::traverse::{
        Crossover,
        CrossoverMethod,
        Mutator,
    };
    pub use crate::genome::Genome;
}

pub fn mutate<G: Genome>(genome: &mut G, rate: f64, rng: &mut impl rand::Rng) {
    let mut mutator = Mutator::new(rate, rng);

    genome.mutate(&mut mutator);
}

pub fn crossover<G: Genome>(
    genome_left: &mut G,
    genome_right: &mut G,
    method: CrossoverMethod,
    rng: &mut impl rand::Rng,
) {
    let method = match method {
        CrossoverMethod::Uniform(rate) => CrossoverState::Uniform(rate),
        CrossoverMethod::KPoint(amount) => CrossoverState::KPoint {
            count: 0,
            length: genome_left.size_hint() as u64,
            swapped: 0,
            desired: amount,
        },
    };
    let mut crossover = Crossover::new(rng, method);

    genome_left.crossover(genome_right, &mut crossover);
}

pub fn reproduce<G: Genome + Clone>(
    parent_left: &G,
    parent_right: &G,
    crossover_method: CrossoverMethod,
    mutation_rate: f64,
    rng: &mut impl rand::Rng,
) -> (G, G) {
    let (mut child_left, mut child_right) = (parent_left.clone(), parent_right.clone());

    crossover(&mut child_left, &mut child_right, crossover_method, rng);
    mutate(&mut child_left, mutation_rate, rng);
    mutate(&mut child_right, mutation_rate, rng);

    (child_left, child_right)
}
