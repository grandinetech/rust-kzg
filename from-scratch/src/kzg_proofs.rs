use kzg::{G1, P1Affine, G2, Fp, P2Affine, Fr};
use blst::{blst_p1_add_or_double,
           blst_p1s_to_affine,
           blst_scalar,
           blst_scalar_from_fr,
           blst_p1_mult,
           blst_p1s_mult_pippenger,
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
};
use crate::poly_utils::{new_poly_div};
use crate::kzg_types::{FsKZGSettings, FsPoly, FsFr};
use crate::utils::{is_power_of_two, log_2_byte};
use crate::consts::{G2_GENERATOR};
use crate::fft_fr::fft_fr;

pub fn commit_to_poly(out: &mut G1, poly: &FsPoly, kzg_settings: &FsKZGSettings) -> Result<(), String> {
    if poly.coeffs.len() > kzg_settings.secret_g1.len() {
        return Err(String::from("Polynomial is longer than secret g1"));
    }

    g1_linear_combination(out, &kzg_settings.secret_g1, &poly.coeffs, poly.coeffs.len());

    return Ok(());
}

pub fn compute_proof_multi(p: &FsPoly, x0: &FsFr, n: usize, kzg_settings: &FsKZGSettings) -> Result<G1, String> {
    //CHECK(is_power_of_two(n));
    assert!(is_power_of_two(n));
    //poly divisor, q;
    //let mut divisor = Poly::default();
    let mut divisor: FsPoly = FsPoly { coeffs: Vec::default() };
    let mut q: FsPoly = FsPoly { coeffs: Vec::default() };

    // Construct x^n - x0^n = (x - x0.w^0)(x - x0.w^1)...(x - x0.w^(n-1))
    // TRY(new_poly(&divisor, n + 1));

    // -(x0^n)
    let x_pow_n = x0.pow(n);
    //fr_t x_pow_n;

    // fr_negate(&divisor.coeffs[0], &x_pow_n);
    divisor.coeffs[0] = x_pow_n.negate();

    // Zeros
    for _ in 1..n {
        // divisor.coeffs[i] = create_fr_zero();
        divisor.coeffs.push(Fr::zero());
    }

    // x^n
    divisor.coeffs.push(Fr::one());
    // divisor.coeffs[n] = create_fr_one();

    // Calculate q = p / (x^n - x0^n)
    //TRY(new_poly_div(&q, p, &divisor));


    let result = new_poly_div(&p, &divisor);
    assert!(result.is_ok());
    q = result.unwrap();

    //TRY(commit_to_poly(out, &q, ks));
    let mut out = G1::default();
    commit_to_poly(&mut out, &q, &kzg_settings).unwrap();

    return Ok(out);
}

pub fn g1_mul(out: &mut G1, a: &G1, b: &FsFr) {
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
        let g1_identity: G1 = G1 {
            x: Fp { l: [0u64; 6] },
            y: Fp { l: [0u64; 6] },
            z: Fp { l: [0u64; 6] },
        };
        *out = g1_identity;
    } else if i == 1 && scalar.b[0] == 1 {
        *out = *a;
    } else {
        // Count the number of bits to be multiplied.
        unsafe {
            blst_p1_mult(out, a, &(scalar.b[0]), 8 * i - 7 + log_2_byte(scalar.b[i - 1]));
        }
    }
}

fn g1_linear_combination(out: &mut G1, p: &Vec<G1>, coeffs: &Vec<FsFr>, len: usize) {
    if len < 8 { // Tunable parameter: must be at least 2 since Blst fails for 0 or 1
        // Direct approach
        let mut tmp: G1 = G1::default();

        let g1_identity: G1 = G1 {
            x: Fp { l: [0u64; 6] },
            y: Fp { l: [0u64; 6] },
            z: Fp { l: [0u64; 6] },
        };

        *out = g1_identity;
        for i in 0..len {
            unsafe {
                g1_mul(&mut tmp, &p[i], &coeffs[i]);
                blst_p1_add_or_double(out, out, &tmp);
            }
        }
    } else {

        // Blst's implementation of the Pippenger method
        //blst_p1_affine *p_affine = malloc(len * sizeof(blst_p1_affine));
        let mut p_affine = vec![P1Affine::default(); len];
        //blst_scalar *scalars = malloc(len * sizeof(blst_scalar));
        let mut scalars = vec![blst_scalar::default(); len];

        // Transform the points to affine representation
        //const blst_p1 *p_arg[2] = {p, NULL};
        // let p_arg: const* = {p, null}
        let p_arg: [*const G1; 2] = [&p[0], &G1::default()];
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

        // Call the Pippenger implementation
        //const byte *scalars_arg[2] = {(byte *)scalars, NULL};
        let scalars_arg: [*const blst_scalar; 2] = [scalars.as_ptr(), &blst_scalar::default()];
        //scalars_arg[0] = &scalars;

        //const blst_p1_affine *points_arg[2] = {p_affine, NULL};
        let points_arg: [*const P1Affine; 2] = [p_affine.as_ptr(), &P1Affine::default()];
        //points_arg[0] = &p_affine;

        //void *scratch = malloc(blst_p1s_mult_pippenger_scratch_sizeof(len));
        let mut scratch: u64 = u64::default();
        //blst_p1s_mult_pippenger(out, points_arg, len, scalars_arg, 256, scratch);
        unsafe {
            blst_p1s_mult_pippenger(out, points_arg.as_ptr(), len, scalars_arg.as_ptr() as *const *const u8, 256, &mut scratch);
        }
    }
}


pub fn check_proof_multi(commitment: &G1, proof: &G1, x: &FsFr, ys: &[FsFr], n: usize, kzg_settings: &FsKZGSettings) -> Result<bool, String> {
    if !is_power_of_two(n) {
        return Err(String::from("n is not a power of two")); // fix to error
    }
    //poly interp;
    let mut interp: FsPoly = FsPoly { coeffs: Vec::default() };
    //interp.length = n;
    //fr_t inv_x, inv_x_pow, x_pow;
    // let mut inv_x: FsFr = FsFr::default();
    // let mut inv_x_pow: FsFr = FsFr::default();
    // let mut x_pow: FsFr = FsFr::default();

    //g2_t xn2, xn_minus_yn;
    let mut xn2: G2 = G2::default();
    let mut xn_minus_yn: G2 = G2::default();

    //g1_t is1, commit_minus_interp;
    let mut is1: G1 = G1::default();
    let mut commit_minus_interp: G1 = G1::default();

    //CHECK(is_power_of_two(n));

    // Interpolate at a coset.
    //TRY(new_poly(&interp, n));
    // new_fr_array(&interp, n); // init Fr of len n 

    //TRY(fft_fr(interp.coeffs, ys, true, n, ks->fs));
    interp.coeffs = fft_fr(ys, true, &kzg_settings.fs).unwrap();

    // Because it is a coset, not the subgroup, we have to multiply the polynomial coefficients by x^-i
    // fr_inv(&inv_x, x);
    // unsafe {
    //     blst_fr_eucl_inverse(&mut inv_x as blst_fr, x as blst_fr);
    // }
    let inv_x = x.eucl_inverse();
    let mut inv_x_pow = inv_x.clone();
    //unsafe {
        for i in 1..n {
            // blst_fr_mul(&mut interp.coeffs[i] as *mut blst_fr, &interp.coeffs[i] as blst_fr, &inv_x_pow as blst_fr);
            // blst_fr_mul(&mut inv_x_pow as *mut blst_fr, &inv_x_pow as blst_fr, &inv_x as blst_fr);
            interp.coeffs[i] = interp.coeffs[i].mul(&inv_x_pow);
            inv_x_pow = inv_x_pow.mul(&inv_x_pow);
        }
    //}

    // [x^n]_2
    // fr_inv(&x_pow, &inv_x_pow);
    // unsafe {
    //     blst_fr_eucl_inverse(&mut x_pow as blst_fr, &inv_x_pow as blst_fr);
    // }
    let x_pow = inv_x_pow.eucl_inverse();

    // g2_mul(&xn2, &g2_generator, &x_pow);
    let scalar: blst_scalar = x_pow.get_scalar();
    unsafe {
        // blst_scalar_from_fr(&mut scalar, &x_pow as blst_fr);
        blst_p2_mult(&mut xn2, &G2_GENERATOR, scalar.b.as_ptr() as *const u8, 8 * std::mem::size_of::<blst_scalar>());
    }

    // [s^n - x^n]_2
    //g2_sub(&xn_minus_yn, &ks->secret_g2[n], &xn2);
    let mut b_negative: G2 = xn2.clone();
    unsafe {
        blst_p2_cneg(&mut b_negative, true);
        blst_p2_add_or_double(&mut xn_minus_yn, &kzg_settings.secret_g2[n], &b_negative);
    }

    // [interpolation_polynomial(s)]_1
    // TRY(commit_to_poly(&is1, &interp, ks));
    let result = commit_to_poly(&mut is1, &interp, &kzg_settings);
    assert!(result.is_ok());

    // [commitment - interpolation_polynomial(s)]_1 = [commit]_1 - [interpolation_polynomial(s)]_1
    //g1_sub(&commit_minus_interp, commitment, &is1);
    let mut b_negative: G1 = is1;
    unsafe {
        blst_p1_cneg(&mut b_negative, true);
        blst_p1_add_or_double(&mut commit_minus_interp, commitment, &b_negative);
    }
    return Ok(pairings_verify(&commit_minus_interp, &G2_GENERATOR, proof, &xn_minus_yn).unwrap());
}

fn pairings_verify(a1: &G1, a2: &G2, b1: &G1, b2: &G2) -> Result<bool, String> {
    //blst_fp12 loop0, loop1, gt_point;
    let mut loop0: blst_fp12 = blst_fp12::default();
    let mut loop1: blst_fp12 = blst_fp12::default();
    let mut gt_point: blst_fp12 = blst_fp12::default();

    // blst_p1_affine aa1, bb1;
    let mut aa1: P1Affine = P1Affine::default();
    let mut bb1: P1Affine = P1Affine::default();

    //blst_p2_affine aa2, bb2;
    let mut aa2: P2Affine = P2Affine::default();
    let mut bb2: P2Affine = P2Affine::default();

    // As an optimisation, we want to invert one of the pairings,
    // so we negate one of the points.
    // g1_t a1neg = *a1;
    let mut a1neg: G1 = *a1;
    unsafe {
        blst_p1_cneg(&mut a1neg, true);
        blst_p1_to_affine(&mut aa1, &a1neg);

        blst_p1_to_affine(&mut bb1, b1);
        blst_p2_to_affine(&mut aa2, a2);
        blst_p2_to_affine(&mut bb2, b2);

        blst_miller_loop(&mut loop0, &aa2, &aa1);
        blst_miller_loop(&mut loop1, &bb2, &bb1);

        blst_fp12_mul(&mut gt_point, &loop0, &loop1);
        blst_final_exp(&mut gt_point, &gt_point);

        return Ok(blst_fp12_is_one(&gt_point));
    }
}
