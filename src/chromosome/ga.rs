use std::marker::PhantomData;

use crate::prelude::*;
use genetic_algorithms::traits::{GeneT, GenotypeT};

// pub enum GAMutation {

// }

struct GAWrapper<G, Ch: ?Sized> {
    pub genome: G,
    chromosome: PhantomData<Ch>,
}

impl<G, Ch> Genome for GAWrapper<G, Ch>
where
    G: GenotypeT<Ch>,
    Ch: GeneT + Genome + ?Sized,
{
    fn mutate(&mut self, mutator: &mut Mutator<impl rand::Rng>) {
        mutator.iter(self.genome.get_dna_mut());
    }

    fn crossover(&mut self, other: &mut Self, crossover: &mut Crossover<impl rand::Rng>) {
        crossover.iter(self.genome.get_dna_mut(), other.genome.get_dna_mut());
    }

    fn size_hint(&self) -> usize {
        self.genome.get_dna().len()
    }
}
