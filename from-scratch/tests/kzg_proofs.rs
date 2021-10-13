use blst::{blst_fr_inverse, blst_fr_mul, blst_p2_mult, blst_p1_add_or_double};
use kzg::{Fr, P1, G1, Fp, P1Affine, G2};
use kzg_from_scratch::fft_fr::fft_fr;
use kzg_from_scratch::kzg_types::{KZGSettings, Poly};
use kzg_from_scratch::utils::is_power_of_two;


// should return what ckzg returns using first arg 'out: bool'
// pub fn check_proof_multi(commitment: &G1, proof: &G1, x: &Fr, ys: &Fr, n: usize, ks: KZGSettings) -> bool {
//     if !is_power_of_two(n) {
//         return false; // fix to error
//     }
//     //poly interp;
//     let interp: Poly = Poly::default();
//     interp.length = n;
//     //fr_t inv_x, inv_x_pow, x_pow;
//     let inv_x: Fr = Fr::default();
//     let inv_x_pow: Fr = Fr::default();
//     let x_pow: Fr = Fr::default();
//
//     //g2_t xn2, xn_minus_yn;
//     let xn2: G2 = G2::default();
//     let xn_minus_yn: G2 = G2::default();
//     //g1_t is1, commit_minus_interp;
//     let is1: G1 = G1::default();
//     let commit_minus_interp: G1 = G1::default();
//
//     // Mostly done down to here
//
//     //CHECK(is_power_of_two(n));
//
//     // Interpolate at a coset.
//     //TRY(new_poly(&interp, n));
//     new_fr_array(&interp, n);
//     TRY(fft_fr(interp.coeffs, ys, true, n, ks->fs));
//
//     // Because it is a coset, not the subgroup, we have to multiply the polynomial coefficients by x^-i
//     fr_inv(&inv_x, x);
//     inv_x_pow = inv_x;
//     for (uint64_t i = 1; i < n; i+ +) {
//         fr_mul(&interp.coeffs[i], &interp.coeffs[i], &inv_x_pow);
//         fr_mul(&inv_x_pow, &inv_x_pow, &inv_x);
//     }
//
//     // [x^n]_2
//     fr_inv(&x_pow, &inv_x_pow);
//     g2_mul(&xn2, &g2_generator, &x_pow);
//
//     // [s^n - x^n]_2
//     g2_sub(&xn_minus_yn, &ks->secret_g2[n], &xn2);
//
//     // [interpolation_polynomial(s)]_1
//     TRY(commit_to_poly(&is1, &interp, ks));
//
//     // [commitment - interpolation_polynomial(s)]_1 = [commit]_1 - [interpolation_polynomial(s)]_1
//     g1_sub(&commit_minus_interp, commitment, &is1);
//
//     *out = pairings_verify(&commit_minus_interp, &g2_generator, proof, &xn_minus_yn);
//
//     // free_poly(&interp);
//
//
//     return true;
// }

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