use crate::data_types::{fr::Fr, g2::G2};
use crate::mcl_methods::set_eth_serialization;
use kzg::eip_4844::BYTES_PER_G2;
use kzg::{G2Mul, G2 as CommonG2};

impl CommonG2 for G2 {
    fn generator() -> Self {
        G2::gen()
    }

    fn negative_generator() -> Self {
        G2::G2_NEGATIVE_GENERATOR
    }

    fn add_or_dbl(&mut self, b: &Self) -> Self {
        let mut g2 = G2::zero();
        if self == b {
            G2::dbl(&mut g2, self);
        } else {
            G2::add(&mut g2, self, b);
        }
        g2
    }

    fn dbl(&self) -> Self {
        let mut g2 = G2::zero();
        G2::dbl(&mut g2, self);
        g2
    }

    fn sub(&self, b: &Self) -> Self {
        let mut g2 = G2::zero();
        G2::sub(&mut g2, self, b);
        g2
    }

    fn equals(&self, b: &Self) -> bool {
        G2::eq(self, b)
    }
}

impl G2Mul<Fr> for G2 {
    fn mul(&self, b: &Fr) -> Self {
        let mut g1 = G2::zero();
        G2::mul(&mut g1, self, b);
        g1
    }
}

impl G2 {
    pub fn from_bytes(bytes: &[u8; BYTES_PER_G2]) -> Result<Self, String> {
        set_eth_serialization(1);
        let mut g2 = G2::default();
        if !G2::deserialize(&mut g2, bytes) {
            return Err("failed to deserialize".to_string());
        }
        Ok(g2)
    }
}
