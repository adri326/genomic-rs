use crate::{
    wrapper::{CrossoverWrapper, MutationWrapper},
    Chromosome, Genome,
};
use rand::Rng;

/// A helper struct for performing the mutation operation on genomes.
pub struct Mutator<R: Rng> {
    rate: f64,
    rng: R,
}

impl<R: Rng> Mutator<R> {
    #[inline(always)]
    pub(crate) fn new(rate: f64, rng: R) -> Self {
        Self { rate, rng }
    }

    /// Instructs the mutation helper to mutate a single chromosome.
    #[inline(always)]
    pub fn chromosome<'a, Ch: Chromosome + ?Sized>(
        &'a mut self,
        chromosome: &mut Ch,
    ) -> &'a mut Self {
        chromosome.mutate(self.rate, &mut self.rng);

        self
    }

    /// Instructs the mutation helper to mutate a sub-genome.
    /// This is currently equivalent to calling `genome.mutate(helper)`,
    /// but using this method is more idiomatic and future-proof.
    #[inline(always)]
    pub fn genome<'a, G: Genome + ?Sized>(&'a mut self, genome: &mut G) -> &'a mut Self {
        genome.mutate(self);

        self
    }

    /// Instructs the mutation helper to mutate an iterator of sub-genomes.
    #[inline(always)]
    pub fn iter<'a, 'b, G: Genome + ?Sized + 'b>(
        &'a mut self,
        genomes: impl IntoIterator<Item = &'b mut G>,
    ) -> &'a mut Self {
        genomes.into_iter().for_each(|item| item.mutate(self));

        self
    }

    /// Defines a group of chromosomes that have a lower rate of mutation than the other chromosomes.
    #[inline(always)]
    pub fn multiply_rate<'a, 'b, F: for<'c> FnOnce(&'c mut Self) + 'b>(
        &'a mut self,
        rate_multiplier: f64,
        callback: F,
    ) -> &'a mut Self {
        let new_rate = (self.rate * rate_multiplier).clamp(0.0, 1.0);
        let old_rate = std::mem::replace(&mut self.rate, new_rate);

        callback(self);

        self.rate = old_rate;
        self
    }

    /// Wraps a chromosome in one of the wrapped types defined in [chromosome.rs].
    /// This is now superceded with [Mutator::with].
    #[deprecated(note = "use Mutator::with instead")]
    pub fn wrap_ch<'a, W, T>(&'a mut self, mut wrapper: W, value: &'_ mut T) -> &'a mut Self
    where
        T: From<W> + Clone,
        W: Chromosome,
    {
        wrapper.mutate(self.rate, &mut self.rng);

        *value = wrapper.into();

        self
    }

    /// Wraps a value into a genome wrapper, allowing you to specify the behavior of mutation
    /// without altering the underlying data structure.
    ///
    /// # Examples
    ///
    /// ```
    /// use genomic::prelude::*;
    /// use genomic::wrapper::ReorderGenome;
    ///
    /// struct TravellingSalesman {
    ///     pub order: Vec<u32>,
    /// }
    ///
    /// impl TravellingSalesman {
    ///     pub fn new(nodes: u32) -> Self {
    ///         Self {
    ///             order: (0..nodes).collect()
    ///         }
    ///     }
    /// }
    ///
    /// impl Genome for TravellingSalesman {
    ///     fn mutate(&mut self, mutator: &mut Mutator<impl rand::Rng>) {
    ///         mutator.with(
    ///             &mut ReorderGenome::Swap,
    ///             &mut self.order,
    ///         );
    ///     }
    ///
    ///     fn crossover(&mut self, other: &mut Self, crossover: &mut Crossover<impl rand::Rng>) {
    ///         unimplemented!();
    ///     }
    ///
    ///     fn size_hint(&self) -> usize {
    ///         self.order.len()
    ///     }
    /// }
    /// ```
    pub fn with<'a, 'b, W, G: 'b>(&'a mut self, wrapper: &mut W, value: G) -> &'a mut Self
    where
        W: MutationWrapper<G>,
    {
        wrapper.mutate_with(value, self);

        self
    }

    // TODO: have a group MutationWrapper and deprecate this
    /// Lets you define a set of mutations as mutating a single, virtual chromosome.
    ///
    /// This method mainly exists as a counterpart of [Crossover::group].
    /// A call to this method should only account for a value of `1` in [Genome::size_hint],
    /// no matter how many chromosomes and sub-genomes are mutated within the callback.
    pub fn group<'a, 'b, F: for<'c> FnOnce(&'c mut Self) + 'b>(
        &'a mut self,
        callback: F,
    ) -> &'a mut Self {
        callback(self);

        self
    }

    /// Returns the mutation rate. For most genomes, you won't need to use this method.
    pub fn get_rate(&self) -> f64 {
        self.rate
    }

    /// Returns the rng of the mutator. For most genomes, you won't need to use this method.
    pub fn get_rng(&mut self) -> &mut R {
        &mut self.rng
    }
}

/// A helper struct for performing the crossover operation on genomes.
pub struct Crossover<R: Rng> {
    rng: R,
    method: CrossoverState,
}

/// The crossover type, used for the [crate::crossover] function.
/// This determines how chromosomes of two individuals will be mixed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CrossoverMethod {
    /// Rolls a random number for each chromosome to determine whether it should be swapped or not.
    /// The `f64` determines the rate at which chromosomes will be swapped.
    /// A rate of `1.0` means that half the chromosomes will be swapped, while a rate of `0.0` means that nothing will happen.
    Uniform(f64),

    /// Splits the genome in `k` points (with `k` being passed to this enum variant),
    /// and swaps all of the chromosomes in the even segments, leaving the odd segments as-is.
    KPoint(u64),
    // TODO: add more crossover operators
}

pub(crate) enum CrossoverState {
    Uniform(f64),
    KPoint {
        count: u64,
        length: u64,
        swapped: u64,
        desired: u64,
    },
    Fixed(bool),
}

impl<R: Rng> Crossover<R> {
    #[inline(always)]
    pub(crate) fn new(rng: R, method: CrossoverState) -> Self {
        Self { rng, method }
    }

    fn should_flip(&mut self) -> bool {
        match self.method {
            CrossoverState::Uniform(rate) => self.rng.gen_bool(rate / 2.0),
            CrossoverState::KPoint {
                ref mut count,
                length,
                ref mut swapped,
                desired,
            } => {
                // NOTE: I am unsure whether this method is enough to guarantee that there will be `k` splits
                let rate = (desired - *swapped) as f64 / (length - *count) as f64;

                if *count < length && self.rng.gen_bool(rate.min(1.0)) {
                    *swapped += 1;
                }

                *swapped % 2 == 1
            }
            CrossoverState::Fixed(res) => res,
        }
    }

    /// Instructs the helper to perform its crossover operation on `ch_left` and `ch_right`.
    /// This is the direct equivalent of [Mutator::chromosome].
    #[inline(always)]
    pub fn chromosome<'a, Ch>(&'a mut self, ch_left: &mut Ch, ch_right: &mut Ch) -> &'a mut Self {
        if self.should_flip() {
            std::mem::swap(ch_left, ch_right);
        }

        self
    }

    /// Instructs the helper to perform the crossover operation on a sub-genome.
    ///
    /// This is the direct equivalent of [Mutator::genome].
    #[inline(always)]
    pub fn genome<'a, G: Genome + ?Sized>(
        &'a mut self,
        genome_left: &mut G,
        genome_right: &mut G,
    ) -> &'a mut Self {
        genome_left.crossover(genome_right, self);

        self
    }

    #[inline(always)]
    pub fn with<'a, 'b, W, G: 'b>(
        &'a mut self,
        wrapper: &mut W,
        genome_left: G,
        genome_right: G,
    ) -> &'a mut Self
    where
        W: CrossoverWrapper<G> + 'b,
    {
        wrapper.crossover_with(genome_left, genome_right, self);

        self
    }

    /// Instructs the helper to perform the crossover operation on a list of sub-genomes.
    ///
    /// This is the direct equivalent of [Mutator::iter].
    #[inline(always)]
    pub fn iter<'a, 'b, G: Genome + ?Sized + 'b>(
        &'a mut self,
        genomes_left: impl IntoIterator<Item = &'b mut G>,
        genomes_right: impl IntoIterator<Item = &'b mut G>,
    ) -> &'a mut Self {
        genomes_left
            .into_iter()
            .zip(genomes_right.into_iter())
            .for_each(|(item_left, item_right)| item_left.crossover(item_right, self));

        self
    }

    /// Groups together multiple operations as if it was a single chromosome.
    ///
    /// This means that you can define multiple `Chromosome`s as needing to be mutated all together,
    /// without explicitely bundling them together in your struct.
    ///
    /// As with [Mutator::group], a call to this method should account for exactly `1` chromosome in [Genome::size_hint],
    /// no matter how many operations are performed in the callback.
    pub fn group<'a, 'b, F: for<'c> FnOnce(&'c mut Crossover<&'c mut R>) + 'b>(
        &'a mut self,
        callback: F,
    ) -> &'a mut Self {
        let should_flip = self.should_flip();

        let mut fixed = Crossover {
            rng: &mut self.rng,
            method: CrossoverState::Fixed(should_flip),
        };

        callback(&mut fixed);

        self
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use crate::chromosome::UniformCh;

    use super::*;

    #[test]
    fn test_wrap_ch() {
        struct MyStruct(i32);

        impl Genome for MyStruct {
            fn mutate(&mut self, mutator: &mut Mutator<impl Rng>) {
                mutator.wrap_ch(UniformCh::new(self.0, -1, 1), &mut self.0);
            }

            fn crossover(&mut self, _other: &mut Self, _crossover: &mut Crossover<impl Rng>) {
                unimplemented!()
            }

            fn size_hint(&self) -> usize {
                1
            }
        }

        let mut instance = MyStruct(0);

        let mut rng = rand::thread_rng();
        let mut values = HashSet::new();
        for _ in 0..100 {
            crate::mutate(&mut instance, 1.0, &mut rng);
            values.insert(instance.0);
            assert!(instance.0 >= -1);
            assert!(instance.0 <= 1);
        }
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_group() {
        struct MyStruct(Vec<i32>);

        impl Genome for MyStruct {
            fn mutate(&mut self, mutator: &mut Mutator<impl Rng>) {
                mutator
                    .group(|m| {
                        m.genome(&mut self.0[0..2]);
                    })
                    .group(|m| {
                        m.genome(&mut self.0[2..4]);
                    });
            }

            fn crossover(&mut self, other: &mut Self, crossover: &mut Crossover<impl Rng>) {
                crossover
                    .group(|c| {
                        c.genome(&mut self.0[0..2], &mut other.0[0..2]);
                    })
                    .group(|c| {
                        c.genome(&mut self.0[2..4], &mut other.0[2..4]);
                    });
            }

            fn size_hint(&self) -> usize {
                2
            }
        }

        fn test_with_method(method: CrossoverMethod) {
            let mut rng = rand::thread_rng();
            for _ in 0..100 {
                let mut instance_a = MyStruct(vec![0, 1, 2, 3]);
                let mut instance_b = MyStruct(vec![4, 5, 6, 7]);

                crate::crossover(&mut instance_a, &mut instance_b, method, &mut rng);

                assert_eq!(instance_a.0[1], instance_a.0[0] + 1);
                assert_eq!(instance_b.0[1], instance_b.0[0] + 1);
                assert_eq!(instance_a.0[3], instance_a.0[2] + 1);
                assert_eq!(instance_b.0[3], instance_b.0[2] + 1);
                for i in 0..4 {
                    assert_ne!(instance_a.0[i], instance_b.0[i]);
                }
            }
        }

        test_with_method(CrossoverMethod::Uniform(0.5));
        test_with_method(CrossoverMethod::KPoint(1));
    }
}
