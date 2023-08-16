use crate::{Chromosome, Genome};
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
    pub fn chromosome<'a, Ch: Chromosome>(&'a mut self, chromosome: &mut Ch) -> &'a mut Self {
        chromosome.mutate(self.rate, &mut self.rng);

        self
    }

    /// Instructs the mutation helper to mutate a sub-genome.
    /// This is currently equivalent to calling `genome.mutate(helper)`,
    /// but using this method is more idiomatic and future-proof.
    #[inline(always)]
    pub fn genome<'a, G: Genome>(&'a mut self, genome: &mut G) -> &'a mut Self {
        genome.mutate(self);

        self
    }

    /// Instructs the mutation helper to mutate an iterator of sub-genomes.
    #[inline(always)]
    pub fn iter<'a, 'b, G: Genome + 'b>(
        &'a mut self,
        genomes: impl IntoIterator<Item = &'b mut G>,
    ) -> &'a mut Self {
        genomes.into_iter().for_each(|item| item.mutate(self));

        self
    }

    /// Defines a group of chromosomes that have a lower rate of mutation than the rest.
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

    /// Wraps a chromosome in one of the wrapped types defined in [chromosome.rs]
    pub fn wrap_ch<'a, W, T>(&'a mut self, mut wrapper: W, value: &'_ mut T) -> &'a mut Self
    where
        T: From<W> + Clone,
        W: Chromosome
    {
        wrapper.mutate(self.rate, &mut self.rng);

        *value = wrapper.into();

        self
    }

    // TODO: a `group()` method
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
}

impl<R: Rng> Crossover<R> {
    #[inline(always)]
    pub(crate) fn new(rng: R, method: CrossoverState) -> Self {
        Self { rng, method }
    }

    /// Instructs the helper to perform its crossover operation on `ch_left` and `ch_right`.
    /// This is the direct equivalent of [Mutator::chromosome].
    #[inline(always)]
    pub fn chromosome<'a, Ch>(
        &'a mut self,
        ch_left: &mut Ch,
        ch_right: &mut Ch,
    ) -> &'a mut Self {
        match self.method {
            CrossoverState::Uniform(rate) => {
                if self.rng.gen_bool(rate / 2.0) {
                    std::mem::swap(ch_left, ch_right);
                }
            }
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

                if *swapped % 2 == 1 {
                    std::mem::swap(ch_left, ch_right);
                }
            }
        }

        self
    }

    /// Instructs the helper to perform the crossover operation on a sub-genome.
    ///
    /// This is the direct equivalent of [Mutator::genome].
    #[inline(always)]
    pub fn genome<'a, G: Genome>(
        &'a mut self,
        genome_left: &mut G,
        genome_right: &mut G,
    ) -> &'a mut Self {
        genome_left.crossover(genome_right, self);

        self
    }

    /// Instructs the helper to perform the crossover operation on a list of sub-genomes.
    ///
    /// This is the direct equivalent of [Mutator::iter].
    #[inline(always)]
    pub fn iter<'a, 'b, G: Genome + 'b>(
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
                mutator
                    .wrap_ch(UniformCh::new(self.0, -1, 1), &mut self.0);
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
}
