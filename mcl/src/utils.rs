extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;

use kzg::eip_4844::{hash_to_bls_field, PrecomputationTableManager, BYTES_PER_FIELD_ELEMENT};
use kzg::eth::c_bindings::{Blob, CKZGSettings, CKzgRet};
use kzg::{Fr, G1Mul, G2Mul};

use crate::consts::{G1_GENERATOR, G2_GENERATOR};
use crate::types::fp::MclFp;
use crate::types::fr::MclFr;
use crate::types::g1::{MclG1, FsG1Affine};
use crate::types::g2::MclG2;
use crate::types::kzg_settings::MclKZGSettings;

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

pub(crate) unsafe fn deserialize_blob(blob: *const Blob) -> Result<Vec<MclFr>, CKzgRet> {
    (*blob)
        .bytes
        .chunks(BYTES_PER_FIELD_ELEMENT)
        .map(|chunk| {
            let mut bytes = [0u8; BYTES_PER_FIELD_ELEMENT];
            bytes.copy_from_slice(chunk);
            if let Ok(result) = MclFr::from_bytes(&bytes) {
                Ok(result)
            } else {
                Err(CKzgRet::BadArgs)
            }
        })
        .collect::<Result<Vec<MclFr>, CKzgRet>>()
}

macro_rules! handle_ckzg_badargs {
    ($x: expr) => {
        match $x {
            Ok(value) => value,
            Err(_) => return kzg::eth::c_bindings::CKzgRet::BadArgs,
        }
    };
}

pub(crate) use handle_ckzg_badargs;

pub(crate) static mut PRECOMPUTATION_TABLES: PrecomputationTableManager<
    MclFr,
    MclG1,
    MclFp,
    FsG1Affine,
> = PrecomputationTableManager::new();

pub(crate) fn kzg_settings_to_c(rust_settings: &MclKZGSettings) -> CKZGSettings {
    use kzg::eth::c_bindings::{blst_fp, blst_fp2, blst_fr, blst_p1, blst_p2};

    CKZGSettings {
        roots_of_unity: Box::leak(
            rust_settings
                .fs
                .roots_of_unity
                .iter()
                .map(|r| blst_fr { l: r.to_blst_fr().l } )
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
        .as_mut_ptr(),
        brp_roots_of_unity: Box::leak(
            rust_settings
                .fs
                .brp_roots_of_unity
                .iter()
                .map(|r| blst_fr { l: r.to_blst_fr().l })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
        .as_mut_ptr(),
        reverse_roots_of_unity: Box::leak(
            rust_settings
                .fs
                .reverse_roots_of_unity
                .iter()
                .map(|r| blst_fr { l: r.to_blst_fr().l })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
        .as_mut_ptr(),
        g1_values_monomial: Box::leak(
            rust_settings
                .g1_values_monomial
                .iter()
                .map(|r| blst_p1 {
                    x: blst_fp { l: r.0.x.d },
                    y: blst_fp { l: r.0.y.d },
                    z: blst_fp { l: r.0.z.d },
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
        .as_mut_ptr(),
        g1_values_lagrange_brp: Box::leak(
            rust_settings
                .g1_values_lagrange_brp
                .iter()
                .map(|r| blst_p1 {
                    x: blst_fp { l: r.0.x.d },
                    y: blst_fp { l: r.0.y.d },
                    z: blst_fp { l: r.0.z.d },
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
        .as_mut_ptr(),
        g2_values_monomial: Box::leak(
            rust_settings
                .g2_values_monomial
                .iter()
                .map(|r| blst_p2 {
                    x: blst_fp2 {
                        fp: [blst_fp { l: r.0.x.d[0].d }, blst_fp { l: r.0.x.d[1].d }],
                    },
                    y: blst_fp2 {
                        fp: [blst_fp { l: r.0.y.d[0].d }, blst_fp { l: r.0.y.d[1].d }],
                    },
                    z: blst_fp2 {
                        fp: [blst_fp { l: r.0.z.d[0].d }, blst_fp { l: r.0.z.d[1].d }],
                    },
                })
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
                            .map(|r| blst_p1 {
                                x: blst_fp { l: r.0.x.d },
                                y: blst_fp { l: r.0.y.d },
                                z: blst_fp { l: r.0.z.d },
                            })
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
