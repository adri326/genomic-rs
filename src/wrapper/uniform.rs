use std::ops::RangeInclusive;

use super::*;

// TODO: `num` feature to use the `num` traits instead?
/// A wrapper around scalar values, changing their mutation operator from bit swaps
/// to uniform random walk.
///
/// If `rate` is `1.0`, then the mutated value will become any number between `min` and `max`.
///
/// The [MutationWrapper] trait is implemented for various scalar types on this trait.
///
/// # Example
///
/// ```rust
/// use genomic::prelude::*;
/// use genomic::wrapper::UniformCh;
///
/// struct Rotation {
///     pub r: f32,
///     pub s: f32,
///     pub t: f32,
/// }
///
/// impl Genome for Rotation {
///     fn mutate(&mut self, mutator: &mut Mutator<impl rand::Rng>) {
///         let mut uniform = UniformCh::new(0.0, std::f32::consts::TAU);
///         mutator
///             .with(&mut uniform, &mut self.r)
///             .with(&mut uniform, &mut self.s)
///             .with(&mut uniform, &mut self.t);
///     }
///
///     // ...
///     # fn crossover(&mut self, other: &mut Self, crossover: &mut Crossover<impl rand::Rng>) {
///     #     unimplemented!();
///     # }
///
///     fn size_hint(&self) -> usize {
///         3
///     }
/// }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UniformCh<T> {
    pub min: T,
    pub max: T,
}

impl<T> UniformCh<T> {
    /// Creates a new chromosome, where the scalar value will uniformly be mutated between `min` and `max`.
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }
}

impl<T> From<RangeInclusive<T>> for UniformCh<T> {
    fn from(value: RangeInclusive<T>) -> Self {
        let (min, max) = value.into_inner();

        Self { min, max }
    }
}

impl<T> From<UniformCh<T>> for RangeInclusive<T> {
    fn from(value: UniformCh<T>) -> Self {
        (value.min)..=(value.max)
    }
}

macro_rules! impl_uniform_int {
    ( $type:ty ) => {
        impl MutationWrapper<&mut $type> for UniformCh<$type> {
            fn mutate_with(&mut self, value: &mut $type, mutator: &mut Mutator<impl Rng>) {
                let range = self.max - self.min;
                let range = (range as f64 * mutator.get_rate()) as $type;
                let half_range_low = range / 2;
                let half_range_high = range / 2 + range % 2;

                *value = (*value).clamp(self.min, self.max);

                let range = if *value - self.min <= half_range_low {
                    self.min..=self.min.saturating_add(range).min(self.max)
                } else if self.max - *value <= half_range_high {
                    self.max.saturating_sub(range).max(self.min)..=self.max
                } else {
                    let low = value.saturating_sub(half_range_low).max(self.min);
                    let high = value.saturating_add(half_range_high).min(self.max);
                    low..=high
                };

                *value = mutator.get_rng().gen_range(range);
            }
        }
    };
}

macro_rules! impl_uniform_float {
    ( $type:ty ) => {
        impl MutationWrapper<&mut $type> for UniformCh<$type> {
            fn mutate_with(&mut self, value: &mut $type, mutator: &mut Mutator<impl Rng>) {
                let range = self.max - self.min;
                let range = range * mutator.get_rate() as $type;
                let half_range = range / 2.0;

                let range = if *value - self.min < half_range {
                    self.min..(self.min + range)
                } else if self.max - *value < half_range {
                    (self.max - range)..self.max
                } else {
                    let low = (*value - half_range).max(self.min);
                    let high = (*value + half_range).min(self.max);

                    low..high
                };

                *value = mutator.get_rng().gen_range(range);
            }
        }
    };
}

impl_uniform_int!(u8);
impl_uniform_int!(u16);
impl_uniform_int!(u32);
impl_uniform_int!(u64);
impl_uniform_int!(u128);
impl_uniform_int!(usize);
impl_uniform_int!(i8);
impl_uniform_int!(i16);
impl_uniform_int!(i32);
impl_uniform_int!(i64);
impl_uniform_int!(i128);
impl_uniform_int!(isize);

impl_uniform_float!(f32);
impl_uniform_float!(f64);
