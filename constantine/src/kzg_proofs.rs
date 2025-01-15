extern crate alloc;

use crate::types::fp::CtFp;
use crate::types::g1::CtG1;
use crate::types::{fr::CtFr, g1::CtG1Affine};

use crate::utils::{ptr_transmute, ptr_transmute_mut};

#[cfg(feature = "constantine_msm")]
use constantine_sys as constantine;
#[cfg(feature = "constantine_msm")]
use kzg::G1Affine;

#[cfg(not(feature = "constantine_msm"))]
use kzg::msm::msm_impls::msm;

#[cfg(not(feature = "constantine_msm"))]
use crate::types::g1::CtG1ProjAddAffine;

use kzg::msm::precompute::PrecomputationTable;

use crate::types::g2::CtG2;

use kzg::PairingVerify;

impl PairingVerify<CtG1, CtG2> for CtG1 {
    fn verify(a1: &CtG1, a2: &CtG2, b1: &CtG1, b2: &CtG2) -> bool {
        pairings_verify(a1, a2, b1, b2)
    }
}

pub fn g1_linear_combination(
    out: &mut CtG1,
    points: &[CtG1],
    scalars: &[CtFr],
    len: usize,
    _precomputation: Option<&PrecomputationTable<CtFr, CtG1, CtFp, CtG1Affine>>,
) {
    #[cfg(feature = "constantine_msm")]
    {
        if len == 0 {
            *out = CtG1::default();
            return;
        }

        #[cfg(feature = "parallel")]
        let pool = unsafe {
            constantine::ctt_threadpool_new(constantine_sys::ctt_cpu_get_num_threads_os())
        };

        #[cfg(not(feature = "parallel"))]
        let pool = unsafe { constantine::ctt_threadpool_new(1) };

        unsafe {
            let points_affine_vec = CtG1Affine::into_affines(points);
            let points_transmuted: &[constantine::bls12_381_g1_aff] =
                core::mem::transmute(points_affine_vec.as_slice());

            let frs_transmuted: &[constantine::bls12_381_fr] = core::mem::transmute(scalars);
            constantine::ctt_bls12_381_g1_jac_multi_scalar_mul_fr_coefs_vartime_parallel(
                pool,
                &mut out.0,
                frs_transmuted.as_ptr(),
                points_transmuted.as_ptr(),
                len,
            );
        }

        unsafe {
            constantine::ctt_threadpool_shutdown(pool);
        }
    }

    #[cfg(not(feature = "constantine_msm"))]
    {
        *out = msm::<CtG1, CtFp, CtG1Affine, CtG1ProjAddAffine, CtFr>(
            points,
            scalars,
            len,
            _precomputation,
        );
    }
}

pub fn pairings_verify(a1: &CtG1, a2: &CtG2, b1: &CtG1, b2: &CtG2) -> bool {
    // FIXME: Remove usage of BLST version, though not sure if there's a constantine version of multi miller loop
    let mut aa1 = blst::blst_p1_affine::default();
    let mut bb1 = blst::blst_p1_affine::default();

    let mut aa2 = blst::blst_p2_affine::default();
    let mut bb2 = blst::blst_p2_affine::default();

    // As an optimisation, we want to invert one of the pairings,
    // so we negate one of the points.
    let mut a1neg: CtG1 = *a1;
    unsafe {
        blst::blst_p1_cneg(ptr_transmute_mut(&mut a1neg.0), true);
        blst::blst_p1_to_affine(&mut aa1, ptr_transmute(&a1neg.0));

        blst::blst_p1_to_affine(&mut bb1, ptr_transmute(&b1.0));
        blst::blst_p2_to_affine(&mut aa2, ptr_transmute(&a2.0));
        blst::blst_p2_to_affine(&mut bb2, ptr_transmute(&b2.0));

        let dst = [0u8; 3];
        let mut pairing_blst = blst::Pairing::new(false, &dst);
        pairing_blst.raw_aggregate(&aa2, &aa1);
        pairing_blst.raw_aggregate(&bb2, &bb1);
        let gt_point = pairing_blst.as_fp12().final_exp();

        blst::blst_fp12_is_one(&gt_point)
    }
}
