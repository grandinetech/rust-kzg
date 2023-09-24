extern crate alloc;

#[cfg(not(feature = "parallel"))]
use alloc::vec;
use alloc::vec::Vec;
#[cfg(not(feature = "parallel"))]
use core::ptr;

#[cfg(feature = "parallel")]
use blst::p1_affines;
#[cfg(not(feature = "parallel"))]
use blst::{
    blst_p1s_mult_pippenger, blst_p1s_mult_pippenger_scratch_sizeof, blst_p1s_to_affine, limb_t,
};

use blst::{
    blst_fp12_is_one, blst_p1, blst_p1_affine, blst_p1_cneg, blst_p1_to_affine, blst_p2_affine,
    blst_p2_to_affine, blst_scalar, blst_scalar_from_fr, Pairing,
};

use kzg::{G1Mul, G1};

use crate::types::fr::FsFr;
use crate::types::g1::FsG1;
use crate::types::g2::FsG2;

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
        let points = unsafe { core::slice::from_raw_parts(points.as_ptr() as *const blst_p1, len) };
        let points = p1_affines::from(points);

        let mut scalar_bytes: Vec<u8> = Vec::with_capacity(len * 32);
        for bytes in scalars.iter().map(|b| {
            let mut scalar = blst_scalar::default();

            unsafe { blst_scalar_from_fr(&mut scalar, &b.0) }

            scalar.b
        }) {
            scalar_bytes.extend_from_slice(&bytes);
        }

        let res = points.mult(scalar_bytes.as_slice(), 255);
        *out = FsG1(res)
    }

    #[cfg(not(feature = "parallel"))]
    {
        let mut scratch: Vec<u8>;
        unsafe {
            scratch = vec![0u8; blst_p1s_mult_pippenger_scratch_sizeof(len)];
        }

        let mut p_affine = vec![blst_p1_affine::default(); len];
        let mut p_scalars = vec![blst_scalar::default(); len];

        let p_arg: [*const blst_p1; 2] = [&points[0].0, ptr::null()];
        unsafe {
            blst_p1s_to_affine(p_affine.as_mut_ptr(), p_arg.as_ptr(), len);
        }

        for i in 0..len {
            unsafe { blst_scalar_from_fr(&mut p_scalars[i], &scalars[i].0) };
        }

        let scalars_arg: [*const blst_scalar; 2] = [p_scalars.as_ptr(), ptr::null()];
        let points_arg: [*const blst_p1_affine; 2] = [p_affine.as_ptr(), ptr::null()];
        unsafe {
            blst_p1s_mult_pippenger(
                &mut out.0,
                points_arg.as_ptr(),
                len,
                scalars_arg.as_ptr() as *const *const u8,
                255,
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
