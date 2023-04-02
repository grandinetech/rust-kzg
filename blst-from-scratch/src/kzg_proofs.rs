extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use core::ptr;

use blst::{
    blst_fp12_is_one, blst_p1, blst_p1_affine, blst_p1_cneg, blst_p1_to_affine,
    blst_p1s_mult_pippenger, blst_p1s_mult_pippenger_scratch_sizeof, blst_p1s_to_affine,
    blst_p2_affine, blst_p2_to_affine, blst_scalar, limb_t, Pairing,
};
use kzg::{G1Mul, G1};

use crate::consts::G1_IDENTITY;
use crate::types::fr::FsFr;
use crate::types::g1::FsG1;
use crate::types::g2::FsG2;

pub fn g1_linear_combination(out: &mut FsG1, p: &[FsG1], coeffs: &[FsFr], len: usize) {
    if len < 8 {
        // Direct approach
        let mut tmp;
        *out = G1_IDENTITY;
        for i in 0..len {
            tmp = p[i].mul(&coeffs[i]);
            *out = out.add_or_dbl(&tmp);
        }
    } else {
        let mut scratch: Vec<u8>;
        unsafe {
            scratch = vec![0u8; blst_p1s_mult_pippenger_scratch_sizeof(len) as usize];
        }

        let mut p_affine = vec![blst_p1_affine::default(); len];
        let mut scalars = vec![blst_scalar::default(); len];

        let p_arg: [*const blst_p1; 2] = [&p[0].0, ptr::null()];
        unsafe {
            blst_p1s_to_affine(p_affine.as_mut_ptr(), p_arg.as_ptr(), len);
        }

        for i in 0..len {
            scalars[i] = blst_scalar {
                b: coeffs[i].to_scalar(),
            };
        }

        let scalars_arg: [*const blst_scalar; 2] = [scalars.as_ptr(), ptr::null()];
        let points_arg: [*const blst_p1_affine; 2] = [p_affine.as_ptr(), ptr::null()];
        unsafe {
            blst_p1s_mult_pippenger(
                &mut out.0,
                points_arg.as_ptr(),
                len,
                scalars_arg.as_ptr() as *const *const u8,
                256,
                scratch.as_mut_ptr() as *mut limb_t,
            );
        }
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
