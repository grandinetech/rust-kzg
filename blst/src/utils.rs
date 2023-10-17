extern crate alloc;

use alloc::vec::Vec;

use kzg::{Fr, G1Mul, G2Mul};
use kzg::eip_4844::hash_to_bls_field;

use crate::consts::{G1_GENERATOR, G2_GENERATOR};
use crate::types::g1::FsG1;
use crate::types::g2::FsG2;

pub fn generate_trusted_setup(n: usize, secret: [u8; 32usize]) -> (Vec<FsG1>, Vec<FsG2>) {
    let s = hash_to_bls_field(&secret);
    let mut s_pow = Fr::one();

    let mut s1 = Vec::with_capacity(n);
    let mut s2 = Vec::with_capacity(n);

    for _ in 0..n {
        s1.push(G1_GENERATOR.mul(&s_pow));
        s2.push(G2_GENERATOR.mul(&s_pow));

        s_pow = s_pow.mul(&s);
    }

    (s1, s2)
}
