#[cfg(test)]
mod tests {

    // use kzg::{Fr, G1, G2};
    // use kzg_from_scratch::kzg_types::{FsKZGSettings, FsFFTSettings, FsPoly, create_fr_u64};
    // use kzg_from_scratch::kzg_proofs::{commit_to_poly, compute_proof_multi, check_proof_multi};
    // use blst::{
    //     blst_fr_mul,
    // };

    // #[test]
    // fn proof_multi() {
    //     // Our polynomial: degree 15, 16 coefficients
    //     // uint64_t coeffs[] = {1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13};
    //     let coeffs: [u64; 16usize] = [1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13];

    //     // FFTSettings fs1, fs2;
    //     let fs1: FFTSettings = FFTSettings::from_scale(coeffs.len()).unwrap();
    //     let fs2: FFTSettings = FFTSettings::from_scale(coeffs.len()).unwrap();

    //     //KZGSettings ks1, ks2;
    //     let kzgSettings_1: KZGSettings = KZGSettings::from_scale(coeffs.len()).unwrap();
    //     let kzgSettings_2: KZGSettings = KZGSettings::from_scale(coeffs.len()).unwrap();

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
    //     generate_trusted_setup(s1, s2, &secret, secrets_len); //TODO: create this function


    //     TEST_CHECK(C_KZG_OK == new_fft_settings(&fs1, 4)); // ln_2 of coeffs.len()
    //     TEST_CHECK(C_KZG_OK == new_kzg_settings(&kzgSettings_1, s1, s2, secrets_len, &fs1));

    //     // Commit to the polynomial
    //     // TEST_CHECK(C_KZG_OK == commit_to_poly(&commitment, &p, &ks1));
    //     let result = commit_to_poly(&mut commitment, &p, &kzgSettings_1);
    //     assert!(result.is_ok());

    //     TEST_CHECK(C_KZG_OK == new_fft_settings(&fs2, coset_scale));
    //     TEST_CHECK(C_KZG_OK == new_kzg_settings(&ks2, s1, s2, secrets_len, &fs2));

    //     // Compute proof at the points [x * root_i] 0 <= i < coset_len
    //     // fr_from_uint64(&x, 5431);
    //     x = create_fr_u64(5431);

    //     TEST_CHECK(C_KZG_OK == compute_proof_multi(&proof, &p, &x, coset_len, &kzgSettings_2));

    //     // y_i is the value of the polynomial at each x_i
    //     for i in 0..coset_len {
    //         fr_mul(&tmp, &x, &kzgSettings_2.fs.expanded_roots_of_unity[i]);
    //         eval_poly(&y[i], &p, &tmp);
    //     }

    //     // Verify the proof that the (unknown) polynomial has value y_i at x_i
    //     TEST_CHECK(C_KZG_OK == check_proof_multi(&result, &commitment, &proof, &x, y, coset_len, &kzgSettings_2));
    //     TEST_CHECK(true == result);

    //     // Change a value and check that the proof fails
    //     fr_add(y + coset_len / 2, y + coset_len / 2, &fr_one);
    //     TEST_CHECK(C_KZG_OK == check_proof_multi(&result, &commitment, &proof, &x, y, coset_len, &kzgSettings_2));
    //     TEST_CHECK(false == result);
    // }
}
