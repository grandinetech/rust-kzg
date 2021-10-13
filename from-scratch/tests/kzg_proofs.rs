use kzg::{Fr, P1, G1, Fp, P1Affine, G2};
use kzg_from_scratch::fft_fr::fft_fr;
use kzg_from_scratch::kzg_types::{KZGSettings, Poly};
use kzg_from_scratch::utils::is_power_of_two;
use kzg_from_scratch::consts::{G2_GENERATOR};
use kzg_from_scratch::kzg_proofs::{commit_to_poly};
use blst::{
    blst_fr_inverse,
    blst_fr_mul,
    blst_p2_mult,
    blst_p1_add_or_double,
    blst_fr_eucl_inverse,
    blst_scalar,
    blst_p1_cneg,
    blst_p2_cneg,
    blst_p1_add_or_double,
    blst_p2_add_or_double,
    blst_scalar_from_fr
};

// fn pairings_verify(const g1_t *a1, const g2_t *a2, const g1_t *b1, const g2_t *b2) -> Result<bool, String> {
//     blst_fp12 loop0, loop1, gt_point;
//     blst_p1_affine aa1, bb1;
//     blst_p2_affine aa2, bb2;

//     // As an optimisation, we want to invert one of the pairings,
//     // so we negate one of the points.
//     g1_t a1neg = *a1;
//     blst_p1_cneg(&a1neg, true);

//     blst_p1_to_affine(&aa1, &a1neg);
//     blst_p1_to_affine(&bb1, b1);
//     blst_p2_to_affine(&aa2, a2);
//     blst_p2_to_affine(&bb2, b2);

//     blst_miller_loop(&loop0, &aa2, &aa1);
//     blst_miller_loop(&loop1, &bb2, &bb1);

//     blst_fp12_mul(&gt_point, &loop0, &loop1);
//     blst_final_exp(&gt_point, &gt_point);

//     return blst_fp12_is_one(&gt_point);
// }

// should return what ckzg returns using first arg 'out: bool'
pub fn check_proof_multi(commitment: &G1, proof: &G1, x: &Fr, ys: &[Fr], n: usize, kzg_settings: KZGSettings) -> Result<bool, String> {
    if !is_power_of_two(n) {
        return false; // fix to error
    }
    //poly interp;
    let interp: Poly = Poly { coeffs: Vec::default()};
    //interp.length = n;
    //fr_t inv_x, inv_x_pow, x_pow;
    let inv_x: Fr = Fr::default();
    let inv_x_pow: Fr = Fr::default();
    let x_pow: Fr = Fr::default();

    //g2_t xn2, xn_minus_yn;
    let xn2: G2 = G2::default();
    let xn_minus_yn: G2 = G2::default();

    //g1_t is1, commit_minus_interp;
    let is1: G1 = G1::default();
    let commit_minus_interp: G1 = G1::default();

    // Mostly done down to here

    //CHECK(is_power_of_two(n));

    // Interpolate at a coset.
    //TRY(new_poly(&interp, n));
    // new_fr_array(&interp, n); // init Fr of len n 

    //TRY(fft_fr(interp.coeffs, ys, true, n, ks->fs));
    interp.coeffs = fft_fr(ys, true, &kzg_settings.fs).unwrap();

    // Because it is a coset, not the subgroup, we have to multiply the polynomial coefficients by x^-i
    // fr_inv(&inv_x, x);
    blst_fr_eucl_inverse(&mut inv_x, x);

    inv_x_pow = inv_x.clone();

    for i in 1..n {
        blst_fr_mul(&mut interp.coeffs[i], &interp.coeffs[i], &inv_x_pow);
        blst_fr_mul(&mut inv_x_pow, &inv_x_pow, &inv_x);
    }

    // [x^n]_2
    // fr_inv(&x_pow, &inv_x_pow);
    blst_fr_eucl_inverse(&mut x_pow, &inv_x_pow);

    // g2_mul(&xn2, &g2_generator, &x_pow);
    let scalar: blst_scalar = blst_scalar::default();
    blst_scalar_from_fr(&mut scalar, &x_pow);
    blst_p2_mult(&mut xn2, &G2_GENERATOR, scalar.b, 8 * std::mem::size_of::<blst_scalar>());

    // [s^n - x^n]_2
    //g2_sub(&xn_minus_yn, &ks->secret_g2[n], &xn2);
    let b_negative: G2 = xn2.clone();
    blst_p2_cneg(&mut b_negative, true);
    blst_p2_add_or_double(&mut xn_minus_yn, &kzg_settings.secret_g2[n], &b_negative);

    // [interpolation_polynomial(s)]_1
    // TRY(commit_to_poly(&is1, &interp, ks));
    commit_to_poly(&mut is1, &interp, &kzg_settings);

    // [commitment - interpolation_polynomial(s)]_1 = [commit]_1 - [interpolation_polynomial(s)]_1
    //g1_sub(&commit_minus_interp, commitment, &is1);
    let b_negative: G1 = is1;
    blst_p1_cneg(&mut b_negative, true);
    blst_p1_add_or_double(&mut commit_minus_interp, commitment, &b_negative);

    return Ok(pairings_verify(&commit_minus_interp, &G2_GENERATOR, proof, &xn_minus_yn));

    // free_poly(&interp);
}

/*
C-kzg copied over (below) as a template for rust code


C_KZG_RET new_fr_array(fr_t **x, size_t n) {
    return c_kzg_malloc((void **)x, n * sizeof **x);
}

C_KZG_RET new_poly(poly *out, uint64_t length) {
    out->length = length;
    return new_fr_array(&out->coeffs, length);
}

C_KZG_RET check_proof_multi(bool *out, const g1_t *commitment, const g1_t *proof, const fr_t *x, const fr_t *ys,
    uint64_t n, const KZGSettings *ks) {
    poly interp;
    fr_t inv_x, inv_x_pow, x_pow;
    g2_t xn2, xn_minus_yn;
    g1_t is1, commit_minus_interp;

    CHECK(is_power_of_two(n));

    // Interpolate at a coset.
    TRY(new_poly(&interp, n));
    TRY(fft_fr(interp.coeffs, ys, true, n, ks->fs));

    // Because it is a coset, not the subgroup, we have to multiply the polynomial coefficients by x^-i
    fr_inv(&inv_x, x);
    inv_x_pow = inv_x;
    for (uint64_t i = 1; i < n; i++) {
    fr_mul(&interp.coeffs[i], &interp.coeffs[i], &inv_x_pow);
    fr_mul(&inv_x_pow, &inv_x_pow, &inv_x);
    }

    // [x^n]_2
    fr_inv(&x_pow, &inv_x_pow);
    g2_mul(&xn2, &g2_generator, &x_pow);

    // [s^n - x^n]_2
    g2_sub(&xn_minus_yn, &ks->secret_g2[n], &xn2);

    // [interpolation_polynomial(s)]_1
    TRY(commit_to_poly(&is1, &interp, ks));

    // [commitment - interpolation_polynomial(s)]_1 = [commit]_1 - [interpolation_polynomial(s)]_1
    g1_sub(&commit_minus_interp, commitment, &is1);

    *out = pairings_verify(&commit_minus_interp, &g2_generator, proof, &xn_minus_yn);

    free_poly(&interp);
    return C_KZG_OK;
}
*/