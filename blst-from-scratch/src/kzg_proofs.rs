use kzg::{G1, G1Mul};
use blst::{blst_p1_add_or_double,
           blst_p1s_to_affine,
           blst_scalar,
           blst_scalar_from_fr,
           blst_p1_mult,
           blst_p1s_mult_pippenger,
           blst_p1s_mult_pippenger_scratch_sizeof,
           blst_p2_mult,
           blst_p1_cneg,
           blst_p2_cneg,
           blst_p2_add_or_double,
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
           blst_fp,
};
use crate::kzg_types::{FsFr, FsG1, FsG2};
use crate::utils::{log_2_byte};

pub fn g2_mul(a: &FsG2, b: &FsFr) -> FsG2 {
    let mut scalar = blst_scalar::default();
    let mut out = FsG2::default();

    unsafe {
        blst_scalar_from_fr(&mut scalar, &b.0);
        blst_p2_mult(&mut out.0, &a.0, scalar.b.as_ptr() as *const u8, 8 * std::mem::size_of::<blst_scalar>());
    }

    out
}

pub fn g2_sub(a: &FsG2, b: &FsG2) -> FsG2 {
    let mut out = FsG2::default();
    let mut bneg: FsG2 = *b;

    unsafe {
        blst_p2_cneg(&mut bneg.0, true);
        blst_p2_add_or_double(&mut out.0, &a.0, &bneg.0);
    }
    out
}

pub fn g1_sub(a: &FsG1, b: &FsG1) -> FsG1 {
    let mut out = FsG1::default();
    let mut bneg: FsG1 = *b;

    unsafe {
        blst_p1_cneg(&mut bneg.0, true);
        blst_p1_add_or_double(&mut out.0, &a.0, &bneg.0);
    }
    out
}

pub fn g1_mul(out: &mut FsG1, a: &FsG1, b: &FsFr) {
    let mut scalar: blst_scalar = blst_scalar::default();
    unsafe {
        blst_scalar_from_fr(&mut scalar, &b.0);
    }

    // Count the number of bytes to be multiplied.
    let mut i = scalar.b.len();
    while i != 0 && scalar.b[i - 1] == 0 {
        i -= 1;
    }

    if i == 0 {
        let g1_identity: FsG1 = FsG1 {
            0: blst_p1 {
                x: blst_fp { l: [0u64; 6] },
                y: blst_fp { l: [0u64; 6] },
                z: blst_fp { l: [0u64; 6] },
            },
        };
        *out = g1_identity;
    } else if i == 1 && scalar.b[0] == 1 {
        *out = *a;
    } else {
        // Count the number of bits to be multiplied.
        unsafe {
            blst_p1_mult(
                &mut out.0,
                &a.0,
                &(scalar.b[0]),
                8 * i - 7 + log_2_byte(scalar.b[i - 1]),
            );
        }
    }
}

pub fn g1_linear_combination(out: &mut FsG1, p: &Vec<FsG1>, coeffs: &Vec<FsFr>, len: usize) {
    if len < 8 {
        // Tunable parameter: must be at least 2 since Blst fails for 0 or 1
        // Direct approach
        let mut tmp: FsG1 = FsG1::default();

        let g1_identity = FsG1 {
            0: blst_p1 {
                x: blst_fp { l: [0u64; 6] },
                y: blst_fp { l: [0u64; 6] },
                z: blst_fp { l: [0u64; 6] },
            },
        };

        *out = g1_identity;
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
        //blst_scalar *scalars = malloc(len * sizeof(blst_scalar));
        let mut scalars = vec![blst_scalar::default(); len];

        // Transform the points to affine representation
        //const blst_p1 *p_arg[2] = {p, NULL};
        // let p_arg: const* = {p, null}
        let mut parr: Vec<*const FsG1> = Vec::default();
        for i in 0..p.len() {
            parr.push(&p[i]);
        }

        let p_arg: [*const blst_p1; 2];
        p_arg = [&p[0].0, &blst_p1::default()];
        //p_arg[0] = &p;

        unsafe {
            blst_p1s_to_affine(p_affine.as_mut_ptr(), p_arg.as_ptr(), len);
        }

        // Transform the field elements to 256-bit scalars
        for i in 0..len {
            unsafe {
                blst_scalar_from_fr(&mut scalars[i], &coeffs[i].0);
            }
        }

        // Call the Pippenger g1_mul implementation
        //const byte *scalars_arg[2] = {(byte *)scalars, NULL};
        let scalars_arg: [*const blst_scalar; 2] = [scalars.as_ptr(), &blst_scalar::default()];

        //const blst_p1_affine *points_arg[2] = {p_affine, NULL};
        let points_arg: [*const blst_p1_affine; 2] = [p_affine.as_ptr(), &blst_p1_affine::default()];

        let mut scratch;
        let newarg = scalars_arg.as_ptr() as *const *const u8;
        unsafe {
            scratch = blst_p1s_mult_pippenger_scratch_sizeof(len) as u64;
        }
        unsafe {
            blst_p1s_mult_pippenger(&mut out.0, points_arg.as_ptr(), len, newarg, 256, &mut scratch);
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
