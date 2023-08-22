use super::*;

mod reorder;
pub use reorder::*;

/// In some cases, you may want to alter the behavior of the mutation operation,
/// without having to change the types in your structures.
///
/// The `MutationWrapper` trait lets you do so, by calling `mutator.with(&mut wrapper_instance, &mut self.thing)`.
pub trait MutationWrapper<G> {
    /// The equivalent of [Genome::mutate], with the wrapper added as first parameter.
    fn mutate_with(&mut self, genome: G, mutator: &mut Mutator<impl rand::Rng>);
}

/// Likes [MutationWrapper], this trait lets you alter the behavior of the crossover operation
/// without having to change the structure you are implementing [Genome] on.
pub trait CrossoverWrapper<G> {
    /// The equivalent of [Genome::crossover], with the wrapper added as first parameter.
    fn crossover_with(
        &mut self,
        genome_left: G,
        genome_right: G,
        crossover: &mut Crossover<impl rand::Rng>,
    );
}
