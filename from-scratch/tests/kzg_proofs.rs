#[cfg(test)]
mod tests {
    
    use kzg::{Fr, G1, G2, FFTSettings, Poly};
    use kzg_from_scratch::kzg_types::{FsKZGSettings, FsFFTSettings, FsPoly, FsFr};
    use kzg_from_scratch::consts::{TRUSTED_SETUP_GENERATOR};
    use kzg_from_scratch::utils::{generate_trusted_setup};
    use kzg_from_scratch::kzg_proofs::{
        commit_to_poly,
        compute_proof_multi,
        check_proof_multi
    };

    #[test]
    fn proof_multi() {
        // Our polynomial: degree 15, 16 coefficients
        // uint64_t coeffs[] = {1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13};
        let coeffs: [u64; 16usize] = [1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13];

        // Compute proof at 2^coset_scale points
        //int coset_scale = 3, coset_len = (1 << coset_scale);
        let coset_scale = 3;
        let coset_len = 1 << coset_scale;

        // FsFFTSettings fs1, fs2;
        let fs1 = FsFFTSettings::new(4/* ?????? */).unwrap();
        let fs2 = FsFFTSettings::new(coset_scale).unwrap();

        //FsKZGSettings ks1, ks2;
        // let kzgSettings_1 = FsKZGSettings::new(coeffs.len());
        // let kzgSettings_2 = FsKZGSettings::new(coeffs.len());

        let mut p = FsPoly { coeffs: vec![Fr::default(); 16]};
        
        // g1_t commitment, proof;
        let mut commitment = G1::default();
        //let proof: G1;

        // fr_t x, tmp;
        // let mut x: Fr = FsFr::default();
        let mut tmp = FsFr::default();
        let result: bool;

        // fr_t y[coset_len];
        let mut y: Vec<FsFr> = Vec::default();
        // must be a vec

        // uint64_t secrets_len = coeffs.len() > coset_len ? coeffs.len() + 1 : coset_len + 1;
        let secrets_len: usize = if coeffs.len() > coset_len {coeffs.len() + 1} else {coset_len + 1};

        //g1_t s1[secrets_len];
        //g2_t s2[secrets_len];
        let mut secret1: Vec<G1> = Vec::default();
        let mut secret2: Vec<G2> = Vec::default();

        // Create the polynomial
        //new_poly(&p, coeffs.len());

        for i in 0..coeffs.len() {
            // fr_from_uint64(&p.coeffs[i], coeffs[i]);
            // p.coeffs[i] = create_fr_u64(coeffs[i]);
            p.coeffs[i] = Fr::from_u64(coeffs[i]);
        }

        // Initialise the secrets and data structures
        generate_trusted_setup(&mut secret1, &mut secret2, &TRUSTED_SETUP_GENERATOR, secrets_len);

        // TEST_CHECK(C_KZG_OK == new_fft_settings(&fs1, 4)); // ln_2 of coeffs.len()
        let fs1 = FsFFTSettings::new(4).unwrap();

        // TEST_CHECK(C_KZG_OK == new_kzg_settings(&kzgSettings_1, s1, s2, secrets_len, &fs1));
        let kzg_settings_1 = FsKZGSettings::new(&secret1, &secret2, secrets_len, &fs1);

        // Commit to the polynomial
        // TEST_CHECK(C_KZG_OK == commit_to_poly(&commitment, &p, &ks1));
        let result = commit_to_poly(&mut commitment, &p, &kzg_settings_1);
        assert!(result.is_ok());

        // TEST_CHECK(C_KZG_OK == new_fft_settings(&fs2, coset_scale));
        let fs2 = FsFFTSettings::new(coset_scale).unwrap();

        // TEST_CHECK(C_KZG_OK == new_kzg_settings(&ks2, s1, s2, secrets_len, &fs2));
        let kzg_settings_2 = FsKZGSettings::new(&secret1, &secret2, secrets_len, &fs2);

        // Compute proof at the points [x * root_i] 0 <= i < coset_len
        // fr_from_uint64(&x, 5431);
        // x = create_fr_u64(5431);
        let x = Fr::from_u64(5431);

        // TEST_CHECK(C_KZG_OK == compute_proof_multi(&proof, &p, &x, coset_len, &kzgSettings_2));
        let result = compute_proof_multi(&p, &x, coset_len, &kzg_settings_2);
        assert!(result.is_ok());
        let proof = result.unwrap();

        // y_i is the value of the polynomial at each x_i
        for i in 0..coset_len {
            // fr_mul(&tmp, &x, &kzgSettings_2.fs.expanded_roots_of_unity[i]);
            //eval_poly(&y[i], &p, &tmp);
            tmp = x.mul(&kzg_settings_2.fs.expanded_roots_of_unity[i]);
            y[i] = p.eval(&tmp);
        }

        // Verify the proof that the (unknown) polynomial has value y_i at x_i
        // TEST_CHECK(C_KZG_OK == check_proof_multi(&result, &commitment, &proof, &x, y, coset_len, &kzgSettings_2));
        let result = check_proof_multi(&commitment, &proof, &x, &y, coset_len, &kzg_settings_2); //return through params mayb?
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);

        //TEST_CHECK(true == result);

        // Change a value and check that the proof fails
        //fr_add(y + coset_len / 2, y + coset_len / 2, &fr_one);
        y[coset_len/2] = y[coset_len / 2].add(&Fr::one());

        //TEST_CHECK(C_KZG_OK == check_proof_multi(&result, &commitment, &proof, &x, y, coset_len, &kzgSettings_2));
        let result = check_proof_multi(&commitment, &proof, &x, &y, coset_len, &kzg_settings_2); //return through params mayb?

        //TEST_CHECK(false == result);
        assert_eq!(result.unwrap(), false);
    }
}
