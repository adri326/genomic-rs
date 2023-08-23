use super::*;
use rand::distributions::Distribution;

/// Wraps a scalar type so that only the `bits` least significant bits are mutated.
///
/// For signed integers, the numbers are interpreted as `bits`-long integers in two's complement representation.
/// This means that using `FixedBits { bits: 7 }` with a `i8` is equivalent to mutating a theoretical `i7`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FixedBits {
    pub bits: u8,
}

impl From<u8> for FixedBits {
    fn from(value: u8) -> Self {
        Self { bits: value }
    }
}

impl From<FixedBits> for u8 {
    fn from(value: FixedBits) -> Self {
        value.bits
    }
}

impl FixedBits {
    pub fn new(bits: u8) -> Self {
        Self { bits }
    }
}

macro_rules! impl_fixed_uint {
    ( $type:ty ) => {
        impl MutationWrapper<&mut $type> for FixedBits {
            fn mutate_with(&mut self, value: &mut $type, mutator: &mut Mutator<impl Rng>) {
                let rate = mutator.get_rate();
                debug_assert!(rate <= 1.0);
                debug_assert!(rate >= 0.0);
                let distribution = rand::distributions::Bernoulli::new(rate * 0.5)
                    .expect("`rate` should be between 0.0 and 1.0");

                for (bit, should_flip) in (0..self.bits.min(<$type>::BITS as u8))
                    .zip(distribution.sample_iter(mutator.get_rng()))
                {
                    if should_flip {
                        *value ^= 1 << bit;
                    }
                }
            }
        }
    };
}

macro_rules! impl_fixed_int {
    ( $type:ty ) => {
        impl MutationWrapper<&mut $type> for FixedBits {
            fn mutate_with(&mut self, value: &mut $type, mutator: &mut Mutator<impl Rng>) {
                let rate = mutator.get_rate();
                debug_assert!(rate <= 1.0);
                debug_assert!(rate >= 0.0);
                let distribution = rand::distributions::Bernoulli::new(rate * 0.5)
                    .expect("`rate` should be between 0.0 and 1.0");

                if self.bits != <$type>::BITS as u8 {
                    let depth = 1 << (self.bits - 1);
                    *value = (*value).clamp(-depth, depth - 1);
                }

                for (bit, should_flip) in
                    (0..(self.bits - 1)).zip(distribution.sample_iter(mutator.get_rng()))
                {
                    if should_flip {
                        *value ^= 1 << bit;
                    }
                }

                // Flip the sign bit in a theoretical `self.bits`-sized two's complement integer
                // `(u_n)_2c = -u_{bits-1}*2^{bits-1} + \sum_{i=0}{i<bits-1}{u_i * 2^i}`
                if distribution.sample(mutator.get_rng()) {
                    if self.bits == <$type>::BITS as u8 {
                        *value ^= 1 << (self.bits - 1);
                    } else if *value < 0 {
                        *value += 1 << (self.bits - 1);
                    } else {
                        *value -= 1 << (self.bits - 1);
                    }
                }
            }
        }
    };
}

impl_fixed_uint!(u8);
impl_fixed_uint!(u16);
impl_fixed_uint!(u32);
impl_fixed_uint!(u64);
impl_fixed_uint!(u128);
impl_fixed_uint!(usize);

impl_fixed_int!(i8);
impl_fixed_int!(i16);
impl_fixed_int!(i32);
impl_fixed_int!(i64);
impl_fixed_int!(i128);
impl_fixed_uint!(isize);

// TODO: implement this for floats?
