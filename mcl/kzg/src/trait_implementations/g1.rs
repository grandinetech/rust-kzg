use crate::data_types::g1::is_valid_order;
use crate::data_types::{fr::Fr, g1::G1};
use crate::fk20_fft::{G1_GENERATOR, G1_NEGATIVE_GENERATOR};
use crate::mcl_methods::set_eth_serialization;
use kzg::eip_4844::BYTES_PER_G1;
use kzg::{G1Mul, G1 as CommonG1};

impl CommonG1 for G1 {
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

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        bytes
            .try_into()
            .map_err(|_| {
                format!(
                    "Invalid byte length. Expected {}, got {}",
                    BYTES_PER_G1,
                    bytes.len()
                )
            })
            .and_then(|bytes: &[u8; BYTES_PER_G1]| {
                set_eth_serialization(1);
                let mut g1 = G1::default();
                if !G1::deserialize(&mut g1, bytes) {
                    return Err("failed to deserialize".to_string());
                }
                Ok(g1)
            })
    }

    fn from_hex(hex: &str) -> Result<Self, String> {
        let bytes = hex::decode(&hex[2..]).unwrap();
        Self::from_bytes(&bytes)
    }

    fn to_bytes(&self) -> [u8; 48] {
        set_eth_serialization(1);
        G1::serialize(self).try_into().unwrap()
    }

    fn add_or_dbl(&mut self, b: &Self) -> Self {
        let mut g1 = G1::zero();
        if self == b {
            G1::dbl(&mut g1, self);
        } else {
            G1::add(&mut g1, self, b);
        }
        g1
    }

    fn is_inf(&self) -> bool {
        G1::eq(self, &G1::G1_IDENTITY)
    }

    fn is_valid(&self) -> bool {
        self.is_valid() && is_valid_order(self)
    }

    fn dbl(&self) -> Self {
        let mut g1 = G1::zero();
        G1::dbl(&mut g1, self);
        g1
    }

    fn add(&self, b: &Self) -> Self {
        let mut g1 = G1::zero();
        G1::add(&mut g1, self, b);
        g1
    }

    fn sub(&self, b: &Self) -> Self {
        let mut g1 = G1::zero();
        G1::sub(&mut g1, self, b);
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
