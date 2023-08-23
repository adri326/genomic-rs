use std::marker::PhantomData;

use crate::wrapper::MutationWrapper;
use crate::{prelude::*, wrapper::CrossoverWrapper};
use genetic_algorithms::traits::{GeneT, GenotypeT};

// TODO: other wrappers with reordering

/// A basic wrapper for `genetic_algorithms`'s [`GenotypeT`](genetic_algorithms::traits::GenotypeT),
/// which assumes that `G` implements `GenotypeT` and that its chromosome type `Ch` implements `Chromosome`.
///
/// The mutation operator will call [`Ch::mutate`](Chromosome::mutate) and the crossover operator will behave as usual.
///
/// Because the `GenotypeT` trait doesn't use associated types, you might have to specify the `Ch` type
/// when calling [`GAWrapper::new`] with `GAWrapper::<YourChType>::new()`.
///
/// # Example
///
/// ```rust
/// use genetic_algorithms::traits::*;
/// use genomic::prelude::*;
/// use genomic::ga::GAWrapper;
///
/// #[derive(Debug, Copy, Clone, Default, PartialEq)]
/// pub struct Gene {
///     pub id: i32,
/// }
///
/// impl GeneT for Gene {
///     // ...
///     # fn new() -> Gene {
///     #     return Gene{id: -1};
///     # }
///     # fn get_id(&self) -> &i32 {
///     #     return &self.id;
///     # }
/// }
///
/// impl Chromosome for Gene {
///     fn mutate(&mut self, rate: f64, rng: &mut impl rand::Rng) {
///         self.id = rng.gen_range(0..10);
///     }
/// }
///
/// #[derive(Debug, Clone, Default, PartialEq)]
/// pub struct Genotype<T: GeneT> {
///     pub dna: Vec<T>,
///     pub fitness: f64,
///     pub age: i32,
/// }
/// impl <T: GeneT> GenotypeT<T> for Genotype<T> {
///     // ...
///     # fn get_dna(&self) -> &Vec<T> {
///     #     &self.dna
///     # }
///     # fn get_dna_mut(&mut self) -> &mut Vec<T> {
///     #     &mut self.dna
///     # }
///     # fn get_fitness(&self) -> &f64 {
///     #     return &self.fitness;
///     # }
///     # fn get_fitness_mut(&mut self) -> &mut f64 {
///     #     return &mut self.fitness;
///     # }
///     # fn get_age_mut(&mut self) -> &mut i32 {
///     #     &mut self.age
///     # }
///     # fn get_age(&self) -> &i32 {
///     #     &self.age
///     # }
///     # fn calculate_fitness(&mut self) {
///     #
///     #     self.fitness = 0.0;
///     #     let mut position = 0;
///     #
///     #     for i in &self.dna{
///     #         let fitness = f64::from(i.get_id()*position);
///     #         self.fitness += fitness;
///     #         position += 1;
///     #     }
///     # }
///     # fn new() -> Self {
///     #     return Genotype{
///     #         dna: Vec::new(),
///     #         fitness: 0.0,
///     #         age: 0,
///     #     }
///     # }
/// }
///
/// impl Genome for Genotype<Gene> {
///     fn mutate(&mut self, mutator: &mut Mutator<impl rand::Rng>) {
///         mutator.with(&mut GAWrapper::new(), self);
///     }
///
///     fn crossover(&mut self, other: &mut Self, crossover: &mut Crossover<impl rand::Rng>) {
///         crossover.with(&mut GAWrapper::new(), self, other);
///     }
///
///     fn size_hint(&self) -> usize {
///         GAWrapper::new().size_hint(self)
///     }
/// }
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct GAWrapper<Ch> {
    chromosome_type: PhantomData<Ch>,
}

impl<Ch> GAWrapper<Ch> {
    pub fn new() -> Self {
        Self {
            chromosome_type: PhantomData,
        }
    }

    pub fn size_hint<G: GenotypeT<Ch>>(&self, genome: &G) -> usize
    where
        Ch: GeneT,
    {
        genome.get_dna().len()
    }
}

impl<G, Ch> MutationWrapper<&mut G> for GAWrapper<Ch>
where
    G: GenotypeT<Ch>,
    Ch: GeneT + Chromosome,
{
    fn mutate_with(&mut self, genome: &mut G, mutator: &mut Mutator<impl rand::Rng>) {
        mutator.iter(genome.get_dna_mut());
    }
}

impl<G, Ch> CrossoverWrapper<&mut G> for GAWrapper<Ch>
where
    G: GenotypeT<Ch>,
    Ch: GeneT,
{
    fn crossover_with(
        &mut self,
        genome_left: &mut G,
        genome_right: &mut G,
        crossover: &mut Crossover<impl rand::Rng>,
    ) {
        genome_left
            .get_dna_mut()
            .into_iter()
            .zip(genome_right.get_dna_mut())
            .for_each(|(ch_left, ch_right)| {
                crossover.chromosome(ch_left, ch_right);
            });
    }
}
