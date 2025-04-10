extern crate alloc;

use alloc::vec::Vec;

use kzg::common_utils::log2_pow2;
use kzg::eip_4844::{hash_to_bls_field, PrecomputationTableManager};
use kzg::{FFTSettings, Fr, G1Mul, G2Mul, FFTG1, G1, G2};

use crate::types::fft_settings::MclFFTSettings;
use crate::types::fp::MclFp;
use crate::types::fr::MclFr;
use crate::types::g1::{MclG1, MclG1Affine, MclG1ProjAddAffine};
use crate::types::g2::MclG2;

pub fn generate_trusted_setup(
    n: usize,
    secret: [u8; 32usize],
) -> (Vec<MclG1>, Vec<MclG1>, Vec<MclG2>) {
    let s = hash_to_bls_field(&secret);
    let mut s_pow = Fr::one();

    let mut g1_monomial_values = Vec::with_capacity(n);
    let mut g2_monomial_values = Vec::with_capacity(n);

    for _ in 0..n {
        g1_monomial_values.push(MclG1::generator().mul(&s_pow));
        g2_monomial_values.push(MclG2::generator().mul(&s_pow));

        s_pow = s_pow.mul(&s);
    }

    let s = MclFFTSettings::new(log2_pow2(n)).unwrap();
    let g1_lagrange_values = s.fft_g1(&g1_monomial_values, true).unwrap();

    (g1_monomial_values, g1_lagrange_values, g2_monomial_values)
}

pub(crate) static mut PRECOMPUTATION_TABLES: PrecomputationTableManager<
    MclFr,
    MclG1,
    MclFp,
    MclG1Affine,
    MclG1ProjAddAffine,
> = PrecomputationTableManager::new();
