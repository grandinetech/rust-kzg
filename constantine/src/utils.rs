extern crate alloc;

use alloc::vec::Vec;

use kzg::eip_4844::{
    hash_to_bls_field, Blob, CKZGSettings, PrecomputationTableManager, BYTES_PER_FIELD_ELEMENT,
    C_KZG_RET, C_KZG_RET_BADARGS, FIELD_ELEMENTS_PER_BLOB, FIELD_ELEMENTS_PER_CELL,
    FIELD_ELEMENTS_PER_EXT_BLOB, TRUSTED_SETUP_NUM_G2_POINTS,
};
use kzg::{Fr, G1Mul, G2Mul};

use crate::consts::{G1_GENERATOR, G2_GENERATOR};
use crate::types::fft_settings::CtFFTSettings;
use crate::types::fp::CtFp;
use crate::types::fr::CtFr;
use crate::types::g1::{CtG1, CtG1Affine};
use crate::types::g2::CtG2;
use crate::types::kzg_settings::CtKZGSettings;

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
        s2.push(G1_GENERATOR); // TODO: this should be lagrange form
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

pub(crate) unsafe fn deserialize_blob(blob: *const Blob) -> Result<Vec<CtFr>, C_KZG_RET> {
    (*blob)
        .bytes
        .chunks(BYTES_PER_FIELD_ELEMENT)
        .map(|chunk| {
            let mut bytes = [0u8; BYTES_PER_FIELD_ELEMENT];
            bytes.copy_from_slice(chunk);
            if let Ok(result) = CtFr::from_bytes(&bytes) {
                Ok(result)
            } else {
                Err(C_KZG_RET_BADARGS)
            }
        })
        .collect::<Result<Vec<CtFr>, C_KZG_RET>>()
}

pub(crate) fn fft_settings_to_rust(
    c_settings: *const CKZGSettings,
) -> Result<CtFFTSettings, String> {
    let settings = unsafe { &*c_settings };

    let roots_of_unity = unsafe {
        core::slice::from_raw_parts(settings.roots_of_unity, FIELD_ELEMENTS_PER_EXT_BLOB + 1)
            .iter()
            .map(|r| CtFr::from_blst_fr(*r))
            .collect::<Vec<CtFr>>()
    };

    let brp_roots_of_unity = unsafe {
        core::slice::from_raw_parts(settings.brp_roots_of_unity, FIELD_ELEMENTS_PER_EXT_BLOB)
            .iter()
            .map(|r| CtFr::from_blst_fr(*r))
            .collect::<Vec<CtFr>>()
    };

    let reverse_roots_of_unity = unsafe {
        core::slice::from_raw_parts(
            settings.reverse_roots_of_unity,
            FIELD_ELEMENTS_PER_EXT_BLOB + 1,
        )
        .iter()
        .map(|r| CtFr::from_blst_fr(*r))
        .collect::<Vec<CtFr>>()
    };

    Ok(CtFFTSettings {
        max_width: FIELD_ELEMENTS_PER_EXT_BLOB,
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

pub(crate) fn kzg_settings_to_rust(c_settings: &CKZGSettings) -> Result<CtKZGSettings, String> {
    Ok(CtKZGSettings {
        fs: fft_settings_to_rust(c_settings)?,
        g1_values_monomial: unsafe {
            core::slice::from_raw_parts(c_settings.g1_values_monomial, FIELD_ELEMENTS_PER_BLOB)
        }
        .iter()
        .map(|r| CtG1::from_blst_p1(*r))
        .collect::<Vec<_>>(),
        g1_values_lagrange_brp: unsafe {
            core::slice::from_raw_parts(c_settings.g1_values_lagrange_brp, FIELD_ELEMENTS_PER_BLOB)
        }
        .iter()
        .map(|r| CtG1::from_blst_p1(*r))
        .collect::<Vec<_>>(),
        g2_values_monomial: unsafe {
            core::slice::from_raw_parts(c_settings.g2_values_monomial, TRUSTED_SETUP_NUM_G2_POINTS)
        }
        .iter()
        .map(|r| CtG2::from_blst_p2(*r))
        .collect::<Vec<_>>(),
        x_ext_fft_columns: unsafe {
            core::slice::from_raw_parts(
                c_settings.x_ext_fft_columns,
                2 * ((FIELD_ELEMENTS_PER_EXT_BLOB / 2) / FIELD_ELEMENTS_PER_CELL),
            )
        }
        .iter()
        .map(|it| {
            unsafe { core::slice::from_raw_parts(*it, FIELD_ELEMENTS_PER_CELL) }
                .iter()
                .map(|it| CtG1::from_blst_p1(*it))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>(),
        precomputation: unsafe { PRECOMPUTATION_TABLES.get_precomputation(c_settings) },
    })
}

pub(crate) fn kzg_settings_to_c(rust_settings: &CtKZGSettings) -> CKZGSettings {
    CKZGSettings {
        roots_of_unity: Box::leak(
            rust_settings
                .fs
                .roots_of_unity
                .iter()
                .map(|r| r.to_blst_fr())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
        .as_mut_ptr(),
        brp_roots_of_unity: Box::leak(
            rust_settings
                .fs
                .brp_roots_of_unity
                .iter()
                .map(|r| r.to_blst_fr())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
        .as_mut_ptr(),
        reverse_roots_of_unity: Box::leak(
            rust_settings
                .fs
                .reverse_roots_of_unity
                .iter()
                .map(|r| r.to_blst_fr())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
        .as_mut_ptr(),
        g1_values_monomial: Box::leak(
            rust_settings
                .g1_values_monomial
                .iter()
                .map(|r| r.to_blst_p1())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
        .as_mut_ptr(),
        g1_values_lagrange_brp: Box::leak(
            rust_settings
                .g1_values_lagrange_brp
                .iter()
                .map(|r| r.to_blst_p1())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
        .as_mut_ptr(),
        g2_values_monomial: Box::leak(
            rust_settings
                .g2_values_monomial
                .iter()
                .map(|r| r.to_blst_p2())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
        .as_mut_ptr(),
        x_ext_fft_columns: Box::leak(
            rust_settings
                .x_ext_fft_columns
                .iter()
                .map(|r| {
                    Box::leak(
                        r.iter()
                            .map(|it| it.to_blst_p1())
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                    )
                    .as_mut_ptr()
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
        .as_mut_ptr(),
        tables: core::ptr::null_mut(),
        wbits: 0,
        scratch_size: 0,
    }
}
