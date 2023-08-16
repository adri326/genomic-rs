use rand::{distributions::Distribution, Rng};

pub trait Chromosome {
    /// Mutates the chromosome, with `rate` being a number between `0.0` and `1.0`.
    ///
    /// A `rate` of `1.0` means that the chromosome should take a fully random value.
    fn mutate(&mut self, rate: f64, rng: &mut impl Rng);
}

/// A wrapper type around scalar values, changing their mutation operator from bit swaps
/// to uniform random walk.
///
/// If `rate` is `1.0`, then the mutated value will become any number between `min` and `max`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UniformCh<T> {
    pub value: T,
    pub min: T,
    pub max: T,
}

impl<T> UniformCh<T> {
    /// Creates a new chromosome, where `value` will uniformly be mutated between `min` and `max`.
    /// The mutation rate dictates how big of a subset of the values around `value` may be chosen when mutating the chromosome
    pub fn new(value: T, min: T, max: T) -> Self {
        Self { value, min, max }
    }
}

impl<T> From<(T, (T, T))> for UniformCh<T> {
    fn from((value, (min, max)): (T, (T, T))) -> Self {
        Self {
            value,
            min,
            max
        }
    }
}

/// Groups together a genome as if it was a single chromosome
pub struct GroupCh<'a, T> {
    wrapped: &'a mut T,
}

impl<'a, T: crate::Genome> Chromosome for GroupCh<'a, T> {
    fn mutate(&mut self, rate: f64, rng: &mut impl Rng) {
        self.wrapped.mutate(&mut crate::Mutator::new(rate, rng));
    }
}

macro_rules! impl_ch_int {
    ( $type:ty ) => {
        impl Chromosome for $type {
            fn mutate(&mut self, rate: f64, rng: &mut impl Rng) {
                debug_assert!(rate <= 1.0);
                debug_assert!(rate >= 0.0);
                let distribution = rand::distributions::Bernoulli::new(rate * 0.5)
                    .expect("`rate` should be between 0.0 and 1.0");

                for (bit, should_flip) in (0..<$type>::BITS).zip(distribution.sample_iter(rng)) {
                    if should_flip {
                        *self ^= 1 << bit;
                    }
                }
            }
        }

        impl Chromosome for UniformCh<$type> {
            fn mutate(&mut self, rate: f64, rng: &mut impl Rng) {
                let range = self.max - self.min;
                let range = (range as f64 * rate) as $type;
                let half_range_low = range / 2;
                let half_range_high = range / 2 + range % 2;

                self.value = self.value.clamp(self.min, self.max);

                let range = if self.value - self.min <= half_range_low {
                    self.min..=self.min.saturating_add(range).min(self.max)
                } else if self.max - self.value <= half_range_high {
                    self.max.saturating_sub(range).max(self.min)..=self.max
                } else {
                    let low = self.value.saturating_sub(half_range_low).max(self.min);
                    let high = self.value.saturating_add(half_range_high).min(self.max);
                    low..=high
                };

                self.value = rng.gen_range(range);
            }
        }

        impl From<UniformCh<$type>> for $type {
            fn from(value: UniformCh<$type>) -> Self {
                value.value
            }
        }
    };
}

macro_rules! impl_ch_float {
    ( $type:ty ) => {
        impl Chromosome for UniformCh<$type> {
            fn mutate(&mut self, rate: f64, rng: &mut impl Rng) {
                let range = self.max - self.min;
                let range = range * rate as $type;
                let half_range = range / 2.0;

                let range = if self.value - self.min < half_range {
                    self.min..(self.min + range)
                } else if self.max - self.value < half_range {
                    (self.max - range)..self.max
                } else {
                    let low = (self.value - half_range).max(self.min);
                    let high = (self.value + half_range).min(self.max);

                    low..high
                };

                self.value = rng.gen_range(range);
            }
        }

        impl From<UniformCh<$type>> for $type {
            fn from(value: UniformCh<$type>) -> Self {
                value.value
            }
        }
    };
}

impl_ch_int!(u8);
impl_ch_int!(u16);
impl_ch_int!(u32);
impl_ch_int!(u64);
impl_ch_int!(u128);
impl_ch_int!(i8);
impl_ch_int!(i16);
impl_ch_int!(i32);
impl_ch_int!(i64);
impl_ch_int!(i128);

impl_ch_float!(f32);
impl_ch_float!(f64);

impl Chromosome for () {
    /// Does nothing
    fn mutate(&mut self, _rate: f64, _rng: &mut impl Rng) {
        // noop
    }
}

impl<T: Chromosome> Chromosome for Box<T> {
    fn mutate(&mut self, rate: f64, rng: &mut impl Rng) {
        self.as_mut().mutate(rate, rng);
    }
}
