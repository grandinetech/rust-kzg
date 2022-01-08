use crate::data_types::{fr::Fr, g1::G1};
use crate::fk20_fft::{G1_GENERATOR, G1_NEGATIVE_GENERATOR};
use kzg::{G1Mul, G1 as CommonG1};

impl CommonG1 for G1 {
    fn default() -> Self {
        G1::zero()
    }

    fn identity() -> Self {
        G1::G1_IDENTITY
    }

    fn generator() -> Self {
        G1_GENERATOR
    }

    fn negative_generator() -> Self {
        G1_NEGATIVE_GENERATOR
    }

    fn rand() -> Self {
        G1::random()
    }

    fn add_or_dbl(&mut self, b: &Self) -> Self {
        let mut g1 = G1::zero();
        if self == b {
            G1::dbl(&mut g1, self);
        } else {
            unsafe { G1::add(&mut g1, self, b); }
        }
        g1
    }

    fn is_inf(&self) -> bool {
        G1::eq(self, &G1::G1_IDENTITY)
    }

    fn dbl(&self) -> Self {
        let mut g1 = G1::zero();
        G1::dbl(&mut g1, self);
        g1
    }

    fn sub(&self, b: &Self) -> Self {
        let mut g1 = G1::zero();
        unsafe { G1::sub(&mut g1, self, b); }
        g1
    }

    fn equals(&self, b: &Self) -> bool {
        G1::eq(self, b)
    }
}

impl G1Mul<Fr> for G1 {
    fn mul(&self, b: &Fr) -> Self {
        let mut g1 = G1::zero();
        G1::mul(&mut g1, self, b);
        g1
    }
}
