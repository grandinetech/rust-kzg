extern crate alloc;

use crate::types::fp::FsFp;
use crate::types::g1::FsG1;
use crate::types::{fr::FsFr, g1::FsG1Affine};

use crate::types::g1::FsG1ProjAddAffine;

use kzg::msm::{msm_impls::msm, precompute::PrecomputationTable};

use crate::types::g2::FsG2;

use kzg::PairingVerify;

impl PairingVerify<FsG1, FsG2> for FsG1 {
    fn verify(a1: &FsG1, a2: &FsG2, b1: &FsG1, b2: &FsG2) -> bool {
        pairings_verify(a1, a2, b1, b2)
    }
}

pub fn g1_linear_combination(
    out: &mut FsG1,
    points: &[FsG1],
    scalars: &[FsFr],
    len: usize,
    precomputation: Option<&PrecomputationTable<FsFr, FsG1, FsFp, FsG1Affine>>,
) {
    #[cfg(feature = "sppark")]
    {
        use blst::{blst_fr, blst_scalar, blst_scalar_from_fr};
        use kzg::{G1Mul, G1};

        if len < 8 {
            *out = FsG1::default();
            for i in 0..len {
                let tmp = points[i].mul(&scalars[i]);
                out.add_or_dbl_assign(&tmp);
            }

            return;
        }

        let scalars =
            unsafe { alloc::slice::from_raw_parts(scalars.as_ptr() as *const blst_fr, len) };

        let point = if let Some(precomputation) = precomputation {
            rust_kzg_mcl_sppark::multi_scalar_mult_prepared(precomputation.table, scalars)
        } else {
            let affines = kzg::msm::msm_impls::batch_convert::<FsG1, FsFp, FsG1Affine>(&points);
            let affines = unsafe {
                alloc::slice::from_raw_parts(affines.as_ptr() as *const blst_p1_affine, len)
            };
            rust_kzg_mcl_sppark::multi_scalar_mult(&affines[0..len], &scalars)
        };

        *out = FsG1(point);
    }

    #[cfg(not(feature = "sppark"))]
    {
        *out = msm::<FsG1, FsFp, FsG1Affine, FsG1ProjAddAffine, FsFr>(
            points,
            scalars,
            len,
            precomputation,
        );
    }
}

pub fn pairings_verify(a1: &FsG1, a2: &FsG2, b1: &FsG1, b2: &FsG2) -> bool {
    todo!()
}
