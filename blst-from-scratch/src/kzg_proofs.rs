extern crate alloc;

use alloc::vec::Vec;

use blst::{
    blst_fp12_is_one, blst_p1, blst_p1_affine, blst_p1_cneg, blst_p1_to_affine, blst_p2_affine,
    blst_p2_to_affine, p1_affines, Pairing,
};

use crate::types::fr::FsFr;
use crate::types::g1::FsG1;
use crate::types::g2::FsG2;

pub fn g1_linear_combination(out: &mut FsG1, points: &[FsG1], scalars: &[FsFr], len: usize) {
    if len == 0 {
        *out = FsG1::default();
        return;
    }

    let points = unsafe { core::slice::from_raw_parts(points.as_ptr() as *const blst_p1, len) };
    let points = p1_affines::from(points);

    let mut scalar_bytes: Vec<u8> = Vec::with_capacity(len * 32);
    for bytes in scalars.iter().map(|b| b.to_scalar()) {
        scalar_bytes.extend_from_slice(&bytes);
    }

    let res = points.mult(scalar_bytes.as_slice(), 255);
    *out = FsG1(res)
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
