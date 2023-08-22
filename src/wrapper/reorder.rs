use super::*;

/// Re-orders the genes in the wrapped genome, instead of modifying the chromosomes one-by-one.
/// The wrapped genome does not need to implement the `Genome` trait,
/// although it needs to implement `IntoIterator` for its .
#[derive(Clone, Copy, PartialEq)]
pub enum ReorderGenome {
    Swap,
}

impl<'a, G, Ch: 'a> MutationWrapper<G> for ReorderGenome
where
    G: IntoIterator<Item = &'a mut Ch>,
{
    fn mutate_with(&mut self, genome: G, mutator: &mut Mutator<impl rand::Rng>) {
        let iter = genome.into_iter();
        // TODO: do this in-place or re-use the vector across calls?
        let mut vec = iter.collect::<Vec<_>>();
        if vec.len() == 0 {
            return;
        }

        let swaps = ((vec.len() as f64 - 1.0) * mutator.get_rate()) as usize;
        let swaps = if mutator.get_rate() > 0.0 && swaps == 0 {
            1
        } else {
            swaps
        };

        let rng = mutator.get_rng();
        for _ in 0..swaps {
            let mut index_a = rng.gen_range(0..vec.len());
            let mut index_b = rng.gen_range(0..vec.len() - 1);
            if index_b >= index_a {
                index_b += 1;
            } else {
                std::mem::swap(&mut index_a, &mut index_b);
            }

            debug_assert!(index_a < index_b);

            let split = vec.split_at_mut(index_b);
            std::mem::swap(split.0[index_a], split.1[0]);
        }
    }
}

impl ReorderGenome {
    pub fn size_hint<G>(&self, genome: G) -> usize
    where
        G: IntoIterator,
    {
        genome.into_iter().count()
    }
}

#[cfg(test)]
mod test {
    use rand::Rng;
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_reorder_vec() {
        struct ReorderVec {
            values: Vec<u32>,
        }

        impl Genome for ReorderVec {
            fn mutate(&mut self, mutator: &mut Mutator<impl Rng>) {
                mutator.with(&mut ReorderGenome::Swap, &mut self.values);
            }

            fn crossover(&mut self, other: &mut Self, crossover: &mut Crossover<impl Rng>) {
                crossover.iter(&mut self.values, &mut other.values);
            }

            fn size_hint(&self) -> usize {
                ReorderGenome::Swap.size_hint(&self.values)
            }
        }

        let mut instance = ReorderVec {
            values: vec![0, 1, 2, 3],
        };

        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            crate::mutate(&mut instance, 0.25, &mut rng);

            let mut counts = [0; 4];
            for v in instance.values.iter().copied() {
                counts[v as usize] += 1;
            }

            assert!(counts.into_iter().all(|c| c == 1));
        }
    }

    #[test]
    fn test_reorder_boxed_slice() {
        struct ReorderBoxedSlice {
            values: Box<[u32]>,
        }

        impl Genome for ReorderBoxedSlice {
            fn mutate(&mut self, mutator: &mut Mutator<impl Rng>) {
                mutator.with(&mut ReorderGenome::Swap, self.values.as_mut());
            }

            fn crossover(&mut self, other: &mut Self, crossover: &mut Crossover<impl Rng>) {
                crossover.iter(self.values.as_mut(), other.values.as_mut());
            }

            fn size_hint(&self) -> usize {
                ReorderGenome::Swap.size_hint(self.values.as_ref())
            }
        }

        let mut instance = ReorderBoxedSlice {
            values: vec![0, 1, 2, 3].into_boxed_slice(),
        };

        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            crate::mutate(&mut instance, 0.25, &mut rng);

            let mut counts = [0; 4];
            for v in instance.values.iter().copied() {
                counts[v as usize] += 1;
            }

            assert!(counts.into_iter().all(|c| c == 1));
        }
    }

    #[test]
    fn test_reorder_hashmap() {
        struct ReorderBoxedSlice {
            values: HashMap<usize, u32>,
        }

        impl Genome for ReorderBoxedSlice {
            fn mutate(&mut self, mutator: &mut Mutator<impl Rng>) {
                mutator.with(&mut ReorderGenome::Swap, &mut self.values.values_mut());
            }

            fn crossover(&mut self, other: &mut Self, crossover: &mut Crossover<impl Rng>) {
                crossover.iter(self.values.values_mut(), other.values.values_mut());
            }

            fn size_hint(&self) -> usize {
                ReorderGenome::Swap.size_hint(&self.values)
            }
        }

        let mut instance = ReorderBoxedSlice {
            values: HashMap::from([(0, 0), (1, 1), (2, 2), (3, 3)]),
        };

        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            crate::mutate(&mut instance, 0.25, &mut rng);

            let mut counts = [0; 4];
            for v in instance.values.values().copied() {
                counts[v as usize] += 1;
            }

            assert!(counts.into_iter().all(|c| c == 1));
        }
    }
}
