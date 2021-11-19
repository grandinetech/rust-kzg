use kzg::{G1Mul};
use blst::{blst_p1_add_or_double,
           blst_p1s_to_affine,
           blst_scalar,
           blst_p1s_mult_pippenger,
           blst_p1s_mult_pippenger_scratch_sizeof,
           blst_p1_cneg,
           blst_fp12,
           blst_p1_to_affine,
           blst_p2_to_affine,
           blst_miller_loop,
           blst_fp12_mul,
           blst_final_exp,
           blst_fp12_is_one,
           blst_p1,
           blst_p1_affine,
           blst_p2_affine,
};
use crate::kzg_types::{FsFr, FsG1, FsG2};
use crate::consts::G1_IDENTITY;


pub fn g1_linear_combination(out: &mut FsG1, p: &[FsG1], coeffs: &[FsFr], len: usize) {
    if true {
        // Tunable parameter: must be at least 2 since Blst fails for 0 or 1
        // Direct approach

        let mut tmp;
        *out = G1_IDENTITY;
        for i in 0..len {
            tmp = p[i].mul(&coeffs[i]);
            unsafe {
                blst_p1_add_or_double(&mut out.0, &out.0, &tmp.0);
            }
        }
    } else {
        // Blst's implementation of the Pippenger method
        //blst_p1_affine *p_affine = malloc(len * sizeof(blst_p1_affine));
        let mut p_affine = vec![blst_p1_affine::default(); len];

        // Transform the points to affine representation
        //const blst_p1 *p_arg[2] = {p, NULL};
        // let p_arg: const* = {p, null}
        let mut parr: Vec<*const FsG1> = Vec::new();
        for i in 0..p.len() {
            parr.push(&p[i]);
        }

        let p_arg: [*const blst_p1; 2];
        p_arg = [&p[0].0, &blst_p1::default()];

        unsafe {
            blst_p1s_to_affine(p_affine.as_mut_ptr(), p_arg.as_ptr(), len);
        }

        let mut scalars = Vec::new();
        // Transform the field elements to 256-bit scalars
        for i in 0..len {
            scalars.push(blst_scalar { b: coeffs[i].to_scalar() });
        }

        // Call the Pippenger g1_mul implementation
        //const byte *scalars_arg[2] = {(byte *)scalars, NULL};
        let scalars_arg: [*const blst_scalar; 2] = [scalars.as_ptr(), &blst_scalar::default()];

        //const blst_p1_affine *points_arg[2] = {p_affine, NULL};
        let points_arg: [*const blst_p1_affine; 2] = [p_affine.as_ptr(), &blst_p1_affine::default()];

        let mut scratch;
        let new_arg = scalars_arg.as_ptr() as *const *const u8;
        unsafe {
            scratch = blst_p1s_mult_pippenger_scratch_sizeof(len) as u64;
        }
        unsafe {
            blst_p1s_mult_pippenger(&mut out.0, points_arg.as_ptr(), len, new_arg, 256, &mut scratch);
        }
    }
}

pub fn pairings_verify(a1: &FsG1, a2: &FsG2, b1: &FsG1, b2: &FsG2) -> bool {
    let mut loop0 = blst_fp12::default();
    let mut loop1 = blst_fp12::default();
    let mut gt_point = blst_fp12::default();

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

        blst_miller_loop(&mut loop0, &aa2, &aa1);
        blst_miller_loop(&mut loop1, &bb2, &bb1);

        blst_fp12_mul(&mut gt_point, &loop0, &loop1);
        blst_final_exp(&mut gt_point, &gt_point);

        return blst_fp12_is_one(&gt_point);
    }
}
