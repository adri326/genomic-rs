use super::*;

/// Wraps a scalar type so that only the `bits` least significant bits are mutated.
///
/// For signed integers, the numbers are interpreted as `bits`-long integers in two's complement representation.
/// This means that `FixedBits<i8> { value: ..., bits: 7 }` is equivalent to a theoretical `i7` when mutating it.
pub struct FixedBits<T> {
    pub value: T,
    pub bits: u8,
}

impl<T> FixedBits<T> {
    pub fn new(value: T, bits: u8) -> Self {
        Self { value, bits }
    }
}

macro_rules! impl_fixed_uint {
    ( $type:ty ) => {
        impl Chromosome for FixedBits<$type> {
            fn mutate(&mut self, rate: f64, rng: &mut impl Rng) {
                debug_assert!(rate <= 1.0);
                debug_assert!(rate >= 0.0);
                let distribution = rand::distributions::Bernoulli::new(rate * 0.5)
                    .expect("`rate` should be between 0.0 and 1.0");

                for (bit, should_flip) in (0..self.bits).zip(distribution.sample_iter(rng)) {
                    if should_flip {
                        self.value ^= 1 << bit;
                    }
                }
            }
        }

        impl From<FixedBits<$type>> for $type {
            fn from(value: FixedBits<$type>) -> $type {
                value.value
            }
        }
    };
}

macro_rules! impl_fixed_int {
    ( $type:ty ) => {
        impl Chromosome for FixedBits<$type> {
            fn mutate(&mut self, rate: f64, rng: &mut impl Rng) {
                debug_assert!(rate <= 1.0);
                debug_assert!(rate >= 0.0);
                let distribution = rand::distributions::Bernoulli::new(rate * 0.5)
                    .expect("`rate` should be between 0.0 and 1.0");

                if self.bits != <$type>::BITS as u8 {
                    let depth = 1 << (self.bits - 1);
                    self.value = self.value.clamp(-depth, depth - 1);
                }

                for (bit, should_flip) in
                    (0..(self.bits - 1)).zip(distribution.sample_iter(&mut *rng))
                {
                    if should_flip {
                        self.value ^= 1 << bit;
                    }
                }

                // Flip the sign bit in a theoretical `self.bits`-sized two's complement integer
                // `(u_n)_2c = -u_{bits-1}*2^{bits-1} + \sum_{i=0}{i<bits-1}{u_i * 2^i}`
                if distribution.sample(rng) {
                    if self.bits == <$type>::BITS as u8 {
                        self.value ^= 1 << (self.bits - 1);
                    } else if self.value < 0 {
                        self.value += 1 << (self.bits - 1);
                    } else {
                        self.value -= 1 << (self.bits - 1);
                    }
                }
            }
        }

        impl From<FixedBits<$type>> for $type {
            fn from(value: FixedBits<$type>) -> $type {
                value.value
            }
        }
    };
}

impl_fixed_uint!(u8);
impl_fixed_uint!(u16);
impl_fixed_uint!(u32);
impl_fixed_uint!(u64);
impl_fixed_uint!(u128);

impl_fixed_int!(i8);
impl_fixed_int!(i16);
impl_fixed_int!(i32);
impl_fixed_int!(i64);
impl_fixed_int!(i128);

// TODO: implement this for floats?
