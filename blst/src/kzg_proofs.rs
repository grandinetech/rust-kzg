extern crate alloc;

use crate::types::fp::FsFp;
use crate::types::g1::FsG1;
use crate::types::{fr::FsFr, g1::FsG1Affine};

#[cfg(not(feature = "parallel"))]
use crate::types::g1::FsG1ProjAddAffine;

use crate::types::g2::FsG2;
use alloc::vec::Vec;
use blst::{
    blst_fp12_is_one, blst_p1_affine, blst_p1_cneg, blst_p1_to_affine, blst_p2_affine,
    blst_p2_to_affine, blst_scalar, blst_scalar_from_fr, Pairing,
};
use kzg::{G1Mul, PairingVerify, Scalar256, G1};

#[cfg(not(feature = "parallel"))]
use kzg::G1Affine;

impl PairingVerify<FsG1, FsG2> for FsG1 {
    fn verify(a1: &FsG1, a2: &FsG2, b1: &FsG1, b2: &FsG2) -> bool {
        pairings_verify(a1, a2, b1, b2)
    }
}

#[cfg(feature = "parallel")]
use kzg::msm::tilling_parallel_pippinger::{parallel_affine_conv, tiling_parallel_pippinger};

pub fn g1_linear_combination(out: &mut FsG1, points: &[FsG1], scalars: &[FsFr], len: usize) {
    if len < 8 {
        *out = FsG1::default();
        for i in 0..len {
            let tmp = points[i].mul(&scalars[i]);
            *out = out.add_or_dbl(&tmp);
        }
        return;
    }

    #[cfg(feature = "parallel")]
    {
        // Atleast on my machine - performance was *slightly worse* with the parallel version
        // let points = FsG1Affine::into_affines(points);
        let points = parallel_affine_conv::<FsG1, FsFp, FsG1Affine>(points);

        let scalars = scalars
            .iter()
            .map(|b| {
                let mut scalar = blst_scalar::default();
                unsafe { blst_scalar_from_fr(&mut scalar, &b.0) }
                Scalar256::from_u8(&scalar.b)
            })
            .collect::<Vec<_>>();
        *out = tiling_parallel_pippinger(&points, scalars.as_slice());
    }

    #[cfg(not(feature = "parallel"))]
    {
        let ark_points = FsG1Affine::into_affines(points);
        let ark_scalars = {
            scalars
                .iter()
                .take(len)
                .map(|scalar| {
                    let mut blst_scalar = blst_scalar::default();
                    unsafe {
                        blst_scalar_from_fr(&mut blst_scalar, &scalar.0);
                    }
                    Scalar256::from_u8(&blst_scalar.b)
                })
                .collect::<Vec<_>>()
        };
        *out = kzg::msm::arkmsm_msm::VariableBaseMSM::multi_scalar_mul::<
            FsG1,
            FsFp,
            FsG1Affine,
            FsG1ProjAddAffine,
        >(&ark_points, &ark_scalars)
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
