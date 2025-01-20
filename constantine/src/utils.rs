extern crate alloc;

use alloc::vec::Vec;

use kzg::eip_4844::{hash_to_bls_field, PrecomputationTableManager};
use kzg::eth::c_bindings::CKZGSettings;
use kzg::{eth, Fr, G1Mul, G2Mul};

use crate::consts::{G1_GENERATOR, G2_GENERATOR};
use crate::types::fft_settings::CtFFTSettings;
use crate::types::fp::CtFp;
use crate::types::fr::CtFr;
use crate::types::g1::{CtG1, CtG1Affine};
use crate::types::g2::CtG2;

pub fn generate_trusted_setup(
    n: usize,
    secret: [u8; 32usize],
) -> (Vec<CtG1>, Vec<CtG1>, Vec<CtG2>) {
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

pub fn ptr_transmute<T, U>(t: &T) -> *const U {
    assert_eq!(core::mem::size_of::<T>(), core::mem::size_of::<U>());

    t as *const T as *const U
}

pub fn ptr_transmute_mut<T, U>(t: &mut T) -> *mut U {
    assert_eq!(core::mem::size_of::<T>(), core::mem::size_of::<U>());

    t as *mut T as *mut U
}

pub(crate) fn fft_settings_to_rust(
    c_settings: *const CKZGSettings,
) -> Result<CtFFTSettings, String> {
    let settings = unsafe { &*c_settings };

    let roots_of_unity = unsafe {
        core::slice::from_raw_parts(
            settings.roots_of_unity,
            eth::FIELD_ELEMENTS_PER_EXT_BLOB + 1,
        )
        .iter()
        .map(|r| CtFr::from_blst_fr(*r))
        .collect::<Vec<CtFr>>()
    };

    let brp_roots_of_unity = unsafe {
        core::slice::from_raw_parts(
            settings.brp_roots_of_unity,
            eth::FIELD_ELEMENTS_PER_EXT_BLOB,
        )
        .iter()
        .map(|r| CtFr::from_blst_fr(*r))
        .collect::<Vec<CtFr>>()
    };

    let reverse_roots_of_unity = unsafe {
        core::slice::from_raw_parts(
            settings.reverse_roots_of_unity,
            eth::FIELD_ELEMENTS_PER_EXT_BLOB + 1,
        )
        .iter()
        .map(|r| CtFr::from_blst_fr(*r))
        .collect::<Vec<CtFr>>()
    };

    Ok(CtFFTSettings {
        max_width: eth::FIELD_ELEMENTS_PER_EXT_BLOB,
        root_of_unity: roots_of_unity[1],
        roots_of_unity,
        brp_roots_of_unity,
        reverse_roots_of_unity,
    })
}

pub(crate) static mut PRECOMPUTATION_TABLES: PrecomputationTableManager<
    CtFr,
    CtG1,
    CtFp,
    CtG1Affine,
> = PrecomputationTableManager::new();
