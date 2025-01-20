extern crate alloc;

use alloc::vec::Vec;

use kzg::eip_4844::{hash_to_bls_field, PrecomputationTableManager};
use kzg::{Fr, G1Mul, G2Mul};

use crate::consts::{G1_GENERATOR, G2_GENERATOR};
use crate::types::fp::MclFp;
use crate::types::fr::MclFr;
use crate::types::g1::{MclG1, MclG1Affine};
use crate::types::g2::MclG2;

pub fn generate_trusted_setup(
    n: usize,
    secret: [u8; 32usize],
) -> (Vec<MclG1>, Vec<MclG1>, Vec<MclG2>) {
    let s = hash_to_bls_field(&secret);
    let mut s_pow = Fr::one();

    let mut s1 = Vec::with_capacity(n);
    let mut s2 = Vec::with_capacity(n);
    let mut s3 = Vec::with_capacity(n);

    for _ in 0..n {
        s1.push(G1_GENERATOR.mul(&s_pow));
        s2.push(G1_GENERATOR.mul(&s_pow)); // TODO: this should be lagrange form
        s3.push(G2_GENERATOR.mul(&s_pow));

        s_pow = s_pow.mul(&s);
    }

    (s1, s2, s3)
}

pub(crate) static mut PRECOMPUTATION_TABLES: PrecomputationTableManager<
    MclFr,
    MclG1,
    MclFp,
    MclG1Affine,
> = PrecomputationTableManager::new();
