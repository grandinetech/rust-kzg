#[cfg(test)]
mod tests {
    
    use kzg::{Fr, G1, P1Affine, G2, P2Affine};
    use kzg_from_scratch::fft_fr::fft_fr;
    use kzg_from_scratch::kzg_types::{KZGSettings, FFTSettings, Poly, create_fr_u64};
    use kzg_from_scratch::utils::is_power_of_two;
    use kzg_from_scratch::consts::{G2_GENERATOR};
    use kzg_from_scratch::kzg_proofs::{commit_to_poly};
    use blst::{
        blst_fr_mul,
        blst_p2_mult,
        blst_p1_add_or_double,
        blst_fr_eucl_inverse,
        blst_scalar,
        blst_p1_cneg,
        blst_p2_cneg,
        blst_p2_add_or_double,
        blst_scalar_from_fr,
        blst_fp12,
        blst_p1_to_affine,
        blst_p2_to_affine,
        blst_miller_loop,
        blst_fp12_mul,
        blst_final_exp,
        blst_fp12_is_one
    };

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

    // should return what ckzg returns using first arg 'out: bool'
    // #[test]
    // fn proof_multi() {
    //     // Our polynomial: degree 15, 16 coefficients
    //     // uint64_t coeffs[] = {1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13};
    //     let coeffs: [u64; 16usize] = [1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13];

    //     // FFTSettings fs1, fs2;
    //     let fs1: FFTSettings = FFTSettings::from_scale(coeffs.len()).unwrap();
    //     let fs2: FFTSettings = FFTSettings::from_scale(coeffs.len()).unwrap();

    //     //KZGSettings ks1, ks2;
    //     let ks1: KZGSettings = KZGSettings::from_scale(coeffs.len()).unwrap();
    //     let ks2: KZGSettings = KZGSettings::from_scale(coeffs.len()).unwrap();

    //     let mut p: Poly = Poly { coeffs: vec![Fr::default(); 16]};
        
    //     // g1_t commitment, proof;
    //     let commitment: G1;
    //     let proof: G1;

    //     // fr_t x, tmp;
    //     let mut x: Fr = Fr::default();
    //     let mut tmp: Fr = Fr::default();
    //     let result: bool;

    //     // Compute proof at 2^coset_scale points
    //     //int coset_scale = 3, coset_len = (1 << coset_scale);
    //     let coset_scale = 3;
    //     let coset_len = 1 << coset_scale;

    //     // fr_t y[coset_len];
    //     let y: Vec<Fr>;
    //     // must be a vec

    //     // uint64_t secrets_len = coeffs.len() > coset_len ? coeffs.len() + 1 : coset_len + 1;
    //     let secrets_len: usize = if coeffs.len() > coset_len {coeffs.len() + 1} else {coset_len + 1};

    //     //g1_t s1[secrets_len];
    //     //g2_t s2[secrets_len];
    //     let secret1: Vec<G1>;
    //     let secret2: Vec<G2>;

    //     // Create the polynomial
    //     //new_poly(&p, coeffs.len());

    //     for i in 0..coeffs.len() {
    //         // fr_from_uint64(&p.coeffs[i], coeffs[i]);
    //         p.coeffs[i] = create_fr_u64(coeffs[i]);
    //     }

    //     // Initialise the secrets and data structures
    //     generate_trusted_setup(s1, s2, &secret, secrets_len);
    //     TEST_CHECK(C_KZG_OK == new_fft_settings(&fs1, 4)); // ln_2 of coeffs.len()
    //     TEST_CHECK(C_KZG_OK == new_kzg_settings(&ks1, s1, s2, secrets_len, &fs1));

    //     // Commit to the polynomial
    //     TEST_CHECK(C_KZG_OK == commit_to_poly(&commitment, &p, &ks1));

    //     TEST_CHECK(C_KZG_OK == new_fft_settings(&fs2, coset_scale));
    //     TEST_CHECK(C_KZG_OK == new_kzg_settings(&ks2, s1, s2, secrets_len, &fs2));

    //     // Compute proof at the points [x * root_i] 0 <= i < coset_len
    //     fr_from_uint64(&x, 5431);
    //     TEST_CHECK(C_KZG_OK == compute_proof_multi(&proof, &p, &x, coset_len, &ks2));

    //     // y_i is the value of the polynomial at each x_i
    //     for i in 0..coset_len {
    //         fr_mul(&tmp, &x, &ks2.fs->expanded_roots_of_unity[i]);
    //         eval_poly(&y[i], &p, &tmp);
    //     }

    //     // Verify the proof that the (unknown) polynomial has value y_i at x_i
    //     TEST_CHECK(C_KZG_OK == check_proof_multi(&result, &commitment, &proof, &x, y, coset_len, &ks2));
    //     TEST_CHECK(true == result);

    //     // Change a value and check that the proof fails
    //     fr_add(y + coset_len / 2, y + coset_len / 2, &fr_one);
    //     TEST_CHECK(C_KZG_OK == check_proof_multi(&result, &commitment, &proof, &x, y, coset_len, &ks2));
    //     TEST_CHECK(false == result);
    // }

    pub fn check_proof_multi(commitment: &G1, proof: &G1, x: &Fr, ys: &[Fr], n: usize, kzg_settings: KZGSettings) -> Result<bool, String> {
        if !is_power_of_two(n) {
            return Err(String::from("n is not a power of two")); // fix to error
        }
        //poly interp;
        let mut interp: Poly = Poly { coeffs: Vec::default()};
        //interp.length = n;
        //fr_t inv_x, inv_x_pow, x_pow;
        let mut inv_x: Fr = Fr::default();
        let mut inv_x_pow: Fr = Fr::default();
        let mut x_pow: Fr = Fr::default();
        
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
        unsafe {
            blst_fr_eucl_inverse(&mut inv_x, x);
        }
        
        inv_x_pow = inv_x.clone();
        unsafe {
            for i in 1..n {
                blst_fr_mul(&mut interp.coeffs[i], &interp.coeffs[i], &inv_x_pow);
                blst_fr_mul(&mut inv_x_pow, &inv_x_pow, &inv_x);
            }
        }
        
        // [x^n]_2
        // fr_inv(&x_pow, &inv_x_pow);
        unsafe {
            blst_fr_eucl_inverse(&mut x_pow, &inv_x_pow);
        }
        
        // g2_mul(&xn2, &g2_generator, &x_pow);
        let mut scalar: blst_scalar = blst_scalar::default();
        unsafe {
            blst_scalar_from_fr(&mut scalar, &x_pow);
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