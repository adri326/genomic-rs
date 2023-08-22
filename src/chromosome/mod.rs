use rand::{distributions::Distribution, Rng};

mod uniform;
pub use uniform::*;

mod fixed;
pub use fixed::*;

#[cfg(feature = "genetic_algorithms")]
pub mod ga;

pub trait Chromosome {
    /// Mutates the chromosome, with `rate` being a number between `0.0` and `1.0`.
    ///
    /// A `rate` of `1.0` means that the chromosome should take a fully random value.
    fn mutate(&mut self, rate: f64, rng: &mut impl Rng);
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

impl Chromosome for () {
    /// Does nothing
    fn mutate(&mut self, _rate: f64, _rng: &mut impl Rng) {
        // noop
    }
}

impl<T: Chromosome + ?Sized> Chromosome for Box<T> {
    fn mutate(&mut self, rate: f64, rng: &mut impl Rng) {
        self.as_mut().mutate(rate, rng);
    }
}

impl Chromosome for bool {
    fn mutate(&mut self, rate: f64, rng: &mut impl Rng) {
        if rng.gen_bool(rate * 0.5) {
            *self = !*self;
        }
    }
}
