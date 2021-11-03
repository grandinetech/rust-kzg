#[cfg(test)]
mod tests {

    use kzg::{Fr, G1, FFTSettings, Poly};
    use kzg_from_scratch::kzg_types::{FsKZGSettings, FsFFTSettings, FsPoly, FsFr, FsG1, FsG2};
    use kzg_from_scratch::consts::{TRUSTED_SETUP_GENERATOR};
    use kzg_from_scratch::utils::{generate_trusted_setup};
    use kzg_from_scratch::kzg_proofs::{commit_to_poly, compute_proof_multi, check_proof_multi, compute_proof_single, check_proof_single};
    use blst::{blst_p1_is_equal, blst_fp, blst_p1};

    // #[test]
    fn _proof_single() {
        // Our polynomial: degree 15, 16 coefficients
        const POLY_LEN: usize = 16;
        let coeffs: [u64; POLY_LEN] = [1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13];
        let secrets_len = POLY_LEN + 1;

        // Create the polynomial
        let mut p: FsPoly = Poly::new(POLY_LEN).unwrap();
        for i in 0..POLY_LEN {
            p.coeffs[i] = Fr::from_u64(coeffs[i]);
        }

        let mut secret1: Vec<FsG1> = Vec::default();
        let mut secret2: Vec<FsG2> = Vec::default();

        // Initialise the secrets and data structures
        generate_trusted_setup(&mut secret1, &mut secret2, &TRUSTED_SETUP_GENERATOR, secrets_len);

        let fs = FsFFTSettings::new(4).unwrap();
        let kzg_settings = FsKZGSettings::new(&secret1, &secret2, secrets_len, &fs);

        // Compute the proof for x = 25
        let x: FsFr = Fr::from_u64(25);
        let mut commitment = G1::default();

        let result = commit_to_poly(&mut commitment, &p, &kzg_settings);
        assert!(result.is_ok());

        let result = compute_proof_single(&p, &x, &kzg_settings);
        assert!(result.is_ok());
        let proof: FsG1 = result.unwrap();

        let mut value: FsFr = p.eval(&x);

        let result: bool = check_proof_single(&commitment, &proof, &x, &value, &kzg_settings);
        assert!(result);

        value = value.add(&FsFr::one());
        let result: bool = check_proof_single(&commitment, &proof, &x, &value, &kzg_settings);
        assert_eq!(false, result);
    }

    // #[test]
    fn _proof_multi() {
        // Our polynomial: degree 15, 16 coefficients
        let coeffs: [u64; 16usize] = [1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13];

        // Compute proof at 2^coset_scale points
        let coset_scale = 3;
        let coset_len = 1 << coset_scale;

        let mut p: FsPoly = Poly::new(coeffs.len()).unwrap();

        let mut commitment = FsG1::default();

        let secrets_len: usize = if coeffs.len() > coset_len {
            coeffs.len() + 1
        } else {
            coset_len + 1
        };

        let mut secret1: Vec<FsG1> = Vec::default();
        let mut secret2: Vec<FsG2> = Vec::default();
        for i in 0..coeffs.len() {
            p.coeffs[i] = Fr::from_u64(coeffs[i]);
        }

        // Initialise the secrets and data structures
        generate_trusted_setup(
            &mut secret1,
            &mut secret2,
            &TRUSTED_SETUP_GENERATOR,
            secrets_len,
        );

        let fs1 = FsFFTSettings::new(4).unwrap();
        let kzg_settings_1 = FsKZGSettings::new(&secret1, &secret2, secrets_len, &fs1);

        // Commit to the polynomial
        let result = commit_to_poly(&mut commitment, &p, &kzg_settings_1);
        assert!(result.is_ok());

        let fs2 = FsFFTSettings::new(coset_scale).unwrap();

        let kzg_settings_2 = FsKZGSettings::new(&secret1, &secret2, secrets_len, &fs2);

        // Compute proof at the points [x * root_i] 0 <= i < coset_len
        let x = Fr::from_u64(5431);

        let result = compute_proof_multi(&p, &x, coset_len, &kzg_settings_2);
        assert!(result.is_ok());
        let proof = result.unwrap();

        let mut y: Vec<FsFr> = Vec::default();
        // y_i is the value of the polynomial at each x_i
        for i in 0..coset_len {
            // fr_mul(&tmp, &x, &kzgSettings_2.fs.expanded_roots_of_unity[i]);
            let tmp: FsFr = x.mul(&kzg_settings_2.fs.expanded_roots_of_unity[i]);
            y.push(p.eval(&tmp));
        }

        // Verify the proof that the (unknown) polynomial has value y_i at x_i
        let result = check_proof_multi(&commitment, &proof, &x, &y, coset_len, &kzg_settings_2);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);

        // Change a value and check that the proof fails
        y[coset_len / 2] = y[coset_len / 2].add(&Fr::one());

        let result = check_proof_multi(&commitment, &proof, &x, &y, coset_len, &kzg_settings_2);
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn commit_to_too_long_poly() {
        let try_fs = FsFFTSettings::new(4);
        assert!(try_fs.is_ok());
        let fs = try_fs.unwrap();

        let secrets_len: usize = 16;
        let poly_len: usize = 32;

        let mut secret_g1: Vec<FsG1> = Vec::default();
        let mut secret_g2: Vec<FsG2> = Vec::default();

        let mut result = G1::default();

        // Initialise the (arbitrary) secrets and data structures
        generate_trusted_setup(&mut secret_g1, &mut secret_g2, &TRUSTED_SETUP_GENERATOR, secrets_len);

        let kzg_settings = FsKZGSettings::new(&secret_g1, &secret_g2, secrets_len, &fs);
        let a = Poly::new(poly_len).unwrap();
        let status = commit_to_poly(&mut result, &a, &kzg_settings);
        assert!(status.is_err());
    }

    #[test]
    fn commit_to_nil_poly() {
        let try_fs = FsFFTSettings::new(4);
        assert!(try_fs.is_ok());
        let fs = try_fs.unwrap();

        let secrets_len: usize = 16;
        let mut secret_g1: Vec<FsG1> = Vec::default();
        let mut secret_g2: Vec<FsG2> = Vec::default();

        let mut result = G1::default();

        // Initialise the (arbitrary) secrets and data structures
        generate_trusted_setup(&mut secret_g1, &mut secret_g2, &TRUSTED_SETUP_GENERATOR, secrets_len);

        let kzg_settings = FsKZGSettings::new(&secret_g1, &secret_g2, secrets_len, &fs);

        let a = Poly::new(0).unwrap();
        let status = commit_to_poly(&mut result, &a, &kzg_settings);
        assert!(status.is_ok());

        let g1_identity: FsG1 = FsG1 {
            0: blst_p1 {
                x: blst_fp { l: [0u64; 6] },
                y: blst_fp { l: [0u64; 6] },
                z: blst_fp { l: [0u64; 6] },
            }
        };

        assert!(g1_equal(&g1_identity, &result));
    }

    pub fn g1_equal(a: &FsG1, b: &FsG1) -> bool {
        unsafe {
            blst_p1_is_equal(&a.0, &b.0)
        }
    }
}
