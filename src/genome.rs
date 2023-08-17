use rand::Rng;

use crate::{Chromosome, Crossover, Mutator};

/// Represents a collection of chromosomes.
///
/// Implementing it means traversing your structure with the `Mutator` and `Crossover` helpers.
/// The implementations of the `mutate` and `crossover` methods should generally be very similar to one another.
///
/// # Example
///
/// ```rust
/// # use genomic::prelude::*;
/// # use genomic::mutate;
///
/// #[derive(Clone, Debug)]
/// struct Pair {
///     left: u32,
///     right: u32,
/// }
///
/// impl Genome for Pair {
///     fn mutate(&mut self, mutator: &mut Mutator<impl rand::Rng>) {
///         mutator
///             .chromosome(&mut self.left)
///             .chromosome(&mut self.right);
///     }
///
///     fn crossover(&mut self, other: &mut Self, crossover: &mut Crossover<impl rand::Rng>) {
///         crossover
///             .chromosome(&mut self.left, &mut other.left)
///             .chromosome(&mut self.right, &mut other.right);
///     }
///
///     fn size_hint(&self) -> usize {
///         2
///     }
/// }
///
/// let mut instance = Pair {
///     left: 0,
///     right: 0,
/// };
///
/// mutate(&mut instance, 1.0, &mut rand::thread_rng());
///
/// let mut instance_copy = instance.clone();
///
/// mutate(&mut instance_copy, 0.1, &mut rand::thread_rng());
///
/// println!("{:?} {:?}", instance, instance_copy);
/// ```
pub trait Genome {
    /// Should mutate the chromosomes contained by this structure, using the `mutator` helper.
    fn mutate(&mut self, mutator: &mut Mutator<impl Rng>);

    /// Should perform a crossover over the chromosomes contained by this structure, using the `crossover` helper.
    ///
    /// **Note:** this function should *not* alter the number of chromosomes contained in either instances.
    /// Doing so might cause functions in this crate to produce incorrect results.
    fn crossover(&mut self, other: &mut Self, crossover: &mut Crossover<impl Rng>);

    /// Should return the number of chromosomes contained in this structure.
    ///
    /// Returning an invalid amount will not crash any of the functions from this crate,
    /// but it might cause them to produce incorrect results.
    fn size_hint(&self) -> usize;
}

impl<Ch: Chromosome> Genome for Ch {
    #[inline(always)]
    fn mutate(&mut self, mutator: &mut Mutator<impl Rng>) {
        mutator.chromosome(self);
    }

    #[inline(always)]
    fn crossover(&mut self, other: &mut Self, crossover: &mut Crossover<impl Rng>) {
        crossover.chromosome(self, other);
    }

    fn size_hint(&self) -> usize {
        1
    }
}

impl<G: Genome> Genome for Vec<G> {
    fn mutate(&mut self, mutator: &mut Mutator<impl Rng>) {
        mutator.iter(self.iter_mut());
    }

    fn crossover(&mut self, other: &mut Self, crossover: &mut Crossover<impl Rng>) {
        crossover.iter(self.iter_mut(), other.iter_mut());
    }

    fn size_hint(&self) -> usize {
        self.iter().map(|item| item.size_hint()).sum()
    }
}

impl<G: Genome> Genome for [G] {
    fn mutate(&mut self, mutator: &mut Mutator<impl Rng>) {
        mutator.iter(self.iter_mut());
    }

    fn crossover(&mut self, other: &mut Self, crossover: &mut Crossover<impl Rng>) {
        crossover.iter(self.iter_mut(), other.iter_mut());
    }

    fn size_hint(&self) -> usize {
        self.iter().map(|item| item.size_hint()).sum()
    }
}

macro_rules! impl_genome_tuple {
    ( $( $name:ident => $id:tt ),+ ) => {
        impl<$($name : Genome),+> Genome for ($($name),+) {
            fn mutate(&mut self, mutator: &mut Mutator<impl Rng>) {
                mutator
                    $(.genome(&mut self.$id))+;
            }

            fn crossover(&mut self, other: &mut Self, crossover: &mut Crossover<impl Rng>) {
                crossover
                    $(.genome(&mut self.$id, &mut other.$id))+;
            }

            fn size_hint(&self) -> usize {
                0
                $(+ self.$id.size_hint())+
            }
        }
    }
}

impl_genome_tuple!(G1 => 0, G2 => 1);
impl_genome_tuple!(G1 => 0, G2 => 1, G3 => 2);
impl_genome_tuple!(G1 => 0, G2 => 1, G3 => 2, G4 => 3);
impl_genome_tuple!(G1 => 0, G2 => 1, G3 => 2, G4 => 3, G5 => 4);
impl_genome_tuple!(G1 => 0, G2 => 1, G3 => 2, G4 => 3, G5 => 4, G6 => 5);
