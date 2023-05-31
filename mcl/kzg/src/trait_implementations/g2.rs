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

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        bytes
            .try_into()
            .map_err(|_| {
                format!(
                    "Invalid byte length. Expected {}, got {}",
                    BYTES_PER_G2,
                    bytes.len()
                )
            })
            .and_then(|bytes: &[u8; BYTES_PER_G2]| {
                set_eth_serialization(1);
                let mut g2 = G2::default();
                if !G2::deserialize(&mut g2, bytes) {
                    return Err("Failed to deserialize".to_string());
                }
                Ok(g2)
            })
    }

    fn to_bytes(&self) -> [u8; 96] {
        set_eth_serialization(1);
        G2::serialize(self).try_into().unwrap()
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
