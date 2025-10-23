extern crate alloc;

use alloc::vec::Vec;

use kzg::common_utils::log2_pow2;
use kzg::eip_4844::{hash_to_bls_field, PrecomputationTableManager};
use kzg::{FFTSettings, Fr, G1Mul, G2Mul, FFTG1};

use crate::consts::{G1_GENERATOR, G2_GENERATOR};
use crate::types::fft_settings::FsFFTSettings;
use crate::types::fp::FsFp;
use crate::types::fr::FsFr;
use crate::types::g1::{FsG1, FsG1Affine, FsG1ProjAddAffine};
use crate::types::g2::FsG2;

pub fn generate_trusted_setup(
    n: usize,
    secret: [u8; 32usize],
) -> (Vec<FsG1>, Vec<FsG1>, Vec<FsG2>) {
    let s = hash_to_bls_field(&secret);
    let mut s_pow = Fr::one();

    let mut g1_monomial_values = Vec::with_capacity(n);
    let mut g2_monomial_values = Vec::with_capacity(n);

    for _ in 0..n {
        g1_monomial_values.push(G1_GENERATOR.mul(&s_pow));
        g2_monomial_values.push(G2_GENERATOR.mul(&s_pow));

        s_pow = s_pow.mul(&s);
    }

    let s = FsFFTSettings::new(log2_pow2(n)).unwrap();
    let g1_lagrange_values = s.fft_g1(&g1_monomial_values, true).unwrap();

    (g1_monomial_values, g1_lagrange_values, g2_monomial_values)
}

pub(crate) static mut PRECOMPUTATION_TABLES: PrecomputationTableManager<
    FsFr,
    FsG1,
    FsFp,
    FsG1Affine,
    FsG1ProjAddAffine,
> = PrecomputationTableManager::new();

#[cfg(feature = "c_bindings")]
#[macro_export]
macro_rules! handle_ckzg_badargs {
    ($x: expr) => {
        match $x {
            Ok(value) => value,
            Err(_) => return kzg::eth::c_bindings::CKzgRet::BadArgs,
        }
    };
}
