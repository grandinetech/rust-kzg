extern crate alloc;

use crate::types::fp::FsFp;
use crate::types::g1::FsG1;
use crate::types::{fr::FsFr, g1::FsG1Affine};

use crate::types::g1::FsG1ProjAddAffine;

use kzg::msm::{msm_impls::msm, precompute::PrecomputationTable};

use crate::types::g2::FsG2;
use blst::{
    blst_fp12_is_one, blst_p1_affine, blst_p1_cneg, blst_p1_to_affine, blst_p2_affine, blst_p2_to_affine, Pairing
};

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
        use kzg::{G1, G1Mul};
        use blst::{blst_fr, blst_scalar, blst_scalar_from_fr};

        if len < 8 {
            *out = FsG1::default();
            for i in 0..len {
                let tmp = points[i].mul(&scalars[i]);
                out.add_or_dbl_assign(&tmp);
            }

            return;
        }

        let scalars = unsafe { alloc::slice::from_raw_parts(scalars.as_ptr() as *const blst_fr, len) };

        let point = if let Some(precomputation) = precomputation {
            rust_kzg_blst_sppark::multi_scalar_mult_prepared(precomputation.table, scalars) 
        } else {
            let affines = kzg::msm::msm_impls::batch_convert::<FsG1, FsFp, FsG1Affine>(&points);
            let affines = unsafe { alloc::slice::from_raw_parts(affines.as_ptr() as *const blst_p1_affine, len) };
            rust_kzg_blst_sppark::multi_scalar_mult(&affines[0..len], &scalars)
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
    let mut aa1 = blst_p1_affine::default();
    let mut bb1 = blst_p1_affine::default();

    let mut aa2 = blst_p2_affine::default();
    let mut bb2 = blst_p2_affine::default();

    // As an optimisation, we want to invert one of the pairings,
    // so we negate one of the points.
    let mut a1neg: FsG1 = *a1;
    unsafe {
        blst_p1_cneg(&mut a1neg.0, true);
        blst_p1_to_affine(&mut aa1, &a1neg.0);

        blst_p1_to_affine(&mut bb1, &b1.0);
        blst_p2_to_affine(&mut aa2, &a2.0);
        blst_p2_to_affine(&mut bb2, &b2.0);

        let dst = [0u8; 3];
        let mut pairing_blst = Pairing::new(false, &dst);
        pairing_blst.raw_aggregate(&aa2, &aa1);
        pairing_blst.raw_aggregate(&bb2, &bb1);
        let gt_point = pairing_blst.as_fp12().final_exp();

        blst_fp12_is_one(&gt_point)
    }
}
