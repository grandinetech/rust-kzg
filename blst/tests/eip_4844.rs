#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::{fs::File, io::Read, ptr::null_mut};

    use kzg::eip_4844::{
        load_trusted_setup_string, Blob, CKZGSettings, KZGCommitment, BYTES_PER_COMMITMENT,
        BYTES_PER_FIELD_ELEMENT, BYTES_PER_G1, BYTES_PER_G2, C_KZG_RET_BADARGS, C_KZG_RET_OK,
    };
    use kzg::Fr;
    use kzg_bench::tests::eip_4844::{
        blob_to_kzg_commitment_test, bytes_to_bls_field_test,
        compute_and_verify_blob_kzg_proof_fails_with_incorrect_proof_test,
        compute_and_verify_blob_kzg_proof_test,
        compute_and_verify_kzg_proof_fails_with_incorrect_proof_test,
        compute_and_verify_kzg_proof_round_trip_test, compute_kzg_proof_test, compute_powers_test,
        generate_random_blob_bytes, get_trusted_setup_path,
        verify_kzg_proof_batch_fails_with_incorrect_proof_test, verify_kzg_proof_batch_test,
    };
    #[cfg(not(feature = "minimal-spec"))]
    use kzg_bench::tests::eip_4844::{
        compute_and_verify_kzg_proof_within_domain_test, test_vectors_blob_to_kzg_commitment,
        test_vectors_compute_blob_kzg_proof, test_vectors_compute_kzg_proof,
        test_vectors_verify_blob_kzg_proof, test_vectors_verify_blob_kzg_proof_batch,
        test_vectors_verify_kzg_proof,
    };
    use rust_kzg_blst::consts::SCALE2_ROOT_OF_UNITY;
    use rust_kzg_blst::eip_4844::{
        blob_to_kzg_commitment, blob_to_kzg_commitment_rust, blob_to_polynomial_rust,
        bytes_to_blob, compute_blob_kzg_proof_rust, compute_kzg_proof_rust, compute_powers,
        evaluate_polynomial_in_evaluation_form_rust, load_trusted_setup,
        load_trusted_setup_filename_rust, verify_blob_kzg_proof_batch_rust,
        verify_blob_kzg_proof_rust, verify_kzg_proof_rust,
    };
    use rust_kzg_blst::types::fft_settings::expand_root_of_unity;
    use rust_kzg_blst::types::{
        fft_settings::FsFFTSettings, fr::FsFr, g1::FsG1, g2::FsG2, kzg_settings::FsKZGSettings,
        poly::FsPoly,
    };

    #[test]
    pub fn bytes_to_bls_field_test_() {
        bytes_to_bls_field_test::<FsFr>();
    }

    #[test]
    pub fn compute_powers_test_() {
        compute_powers_test::<FsFr>(&compute_powers);
    }

    #[test]
    pub fn blob_to_kzg_commitment_test_() {
        blob_to_kzg_commitment_test::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &load_trusted_setup_filename_rust,
            &blob_to_kzg_commitment_rust,
        );
    }

    #[test]
    pub fn compute_kzg_proof_test_() {
        compute_kzg_proof_test::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &load_trusted_setup_filename_rust,
            &compute_kzg_proof_rust,
            &blob_to_polynomial_rust,
            &evaluate_polynomial_in_evaluation_form_rust,
        );
    }

    #[test]
    pub fn compute_and_verify_kzg_proof_round_trip_test_() {
        compute_and_verify_kzg_proof_round_trip_test::<
            FsFr,
            FsG1,
            FsG2,
            FsPoly,
            FsFFTSettings,
            FsKZGSettings,
        >(
            &load_trusted_setup_filename_rust,
            &blob_to_kzg_commitment_rust,
            &bytes_to_blob,
            &compute_kzg_proof_rust,
            &blob_to_polynomial_rust,
            &evaluate_polynomial_in_evaluation_form_rust,
            &verify_kzg_proof_rust,
        );
    }

    #[cfg(not(feature = "minimal-spec"))]
    #[test]
    pub fn compute_and_verify_kzg_proof_within_domain_test_() {
        compute_and_verify_kzg_proof_within_domain_test::<
            FsFr,
            FsG1,
            FsG2,
            FsPoly,
            FsFFTSettings,
            FsKZGSettings,
        >(
            &load_trusted_setup_filename_rust,
            &blob_to_kzg_commitment_rust,
            &bytes_to_blob,
            &compute_kzg_proof_rust,
            &blob_to_polynomial_rust,
            &evaluate_polynomial_in_evaluation_form_rust,
            &verify_kzg_proof_rust,
        );
    }

    #[test]
    pub fn compute_and_verify_kzg_proof_fails_with_incorrect_proof_test_() {
        compute_and_verify_kzg_proof_fails_with_incorrect_proof_test::<
            FsFr,
            FsG1,
            FsG2,
            FsPoly,
            FsFFTSettings,
            FsKZGSettings,
        >(
            &load_trusted_setup_filename_rust,
            &blob_to_kzg_commitment_rust,
            &bytes_to_blob,
            &compute_kzg_proof_rust,
            &blob_to_polynomial_rust,
            &evaluate_polynomial_in_evaluation_form_rust,
            &verify_kzg_proof_rust,
        );
    }

    #[test]
    pub fn compute_and_verify_blob_kzg_proof_test_() {
        compute_and_verify_blob_kzg_proof_test::<
            FsFr,
            FsG1,
            FsG2,
            FsPoly,
            FsFFTSettings,
            FsKZGSettings,
        >(
            &load_trusted_setup_filename_rust,
            &blob_to_kzg_commitment_rust,
            &bytes_to_blob,
            &compute_blob_kzg_proof_rust,
            &verify_blob_kzg_proof_rust,
        );
    }

    #[test]
    pub fn compute_and_verify_blob_kzg_proof_fails_with_incorrect_proof_test_() {
        compute_and_verify_blob_kzg_proof_fails_with_incorrect_proof_test::<
            FsFr,
            FsG1,
            FsG2,
            FsPoly,
            FsFFTSettings,
            FsKZGSettings,
        >(
            &load_trusted_setup_filename_rust,
            &blob_to_kzg_commitment_rust,
            &bytes_to_blob,
            &compute_blob_kzg_proof_rust,
            &verify_blob_kzg_proof_rust,
        );
    }

    #[test]
    pub fn verify_kzg_proof_batch_test_() {
        verify_kzg_proof_batch_test::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &load_trusted_setup_filename_rust,
            &blob_to_kzg_commitment_rust,
            &bytes_to_blob,
            &compute_blob_kzg_proof_rust,
            &verify_blob_kzg_proof_batch_rust,
        );
    }

    #[test]
    pub fn verify_kzg_proof_batch_fails_with_incorrect_proof_test_() {
        verify_kzg_proof_batch_fails_with_incorrect_proof_test::<
            FsFr,
            FsG1,
            FsG2,
            FsPoly,
            FsFFTSettings,
            FsKZGSettings,
        >(
            &load_trusted_setup_filename_rust,
            &blob_to_kzg_commitment_rust,
            &bytes_to_blob,
            &compute_blob_kzg_proof_rust,
            &verify_blob_kzg_proof_batch_rust,
        );
    }

    #[cfg(not(feature = "minimal-spec"))]
    #[test]
    pub fn test_vectors_blob_to_kzg_commitment_() {
        test_vectors_blob_to_kzg_commitment::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &load_trusted_setup_filename_rust,
            &blob_to_kzg_commitment_rust,
            &bytes_to_blob,
        );
    }

    #[cfg(not(feature = "minimal-spec"))]
    #[test]
    pub fn test_vectors_compute_kzg_proof_() {
        test_vectors_compute_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &load_trusted_setup_filename_rust,
            &compute_kzg_proof_rust,
            &bytes_to_blob,
        );
    }

    #[cfg(not(feature = "minimal-spec"))]
    #[test]
    pub fn test_vectors_compute_blob_kzg_proof_() {
        test_vectors_compute_blob_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &load_trusted_setup_filename_rust,
            &bytes_to_blob,
            &compute_blob_kzg_proof_rust,
        );
    }

    #[cfg(not(feature = "minimal-spec"))]
    #[test]
    pub fn test_vectors_verify_kzg_proof_() {
        test_vectors_verify_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &load_trusted_setup_filename_rust,
            &verify_kzg_proof_rust,
        );
    }

    #[cfg(not(feature = "minimal-spec"))]
    #[test]
    pub fn test_vectors_verify_blob_kzg_proof_() {
        test_vectors_verify_blob_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &load_trusted_setup_filename_rust,
            &bytes_to_blob,
            &verify_blob_kzg_proof_rust,
        );
    }

    #[cfg(not(feature = "minimal-spec"))]
    #[test]
    pub fn test_vectors_verify_blob_kzg_proof_batch_() {
        test_vectors_verify_blob_kzg_proof_batch::<
            FsFr,
            FsG1,
            FsG2,
            FsPoly,
            FsFFTSettings,
            FsKZGSettings,
        >(
            &load_trusted_setup_filename_rust,
            &bytes_to_blob,
            &verify_blob_kzg_proof_batch_rust,
        );
    }

    #[test]
    pub fn blob_to_kzg_commitment_invalid_blob() {
        let mut file = File::open(get_trusted_setup_path()).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let (g1_bytes, g2_bytes) = load_trusted_setup_string(&contents).unwrap();

        let mut c_settings = CKZGSettings {
            g1_values: null_mut(),
            g2_values: null_mut(),
            max_width: 0,
            roots_of_unity: null_mut(),
        };

        let status = unsafe {
            load_trusted_setup(
                &mut c_settings,
                g1_bytes.as_ptr(),
                g1_bytes.len() / BYTES_PER_G1,
                g2_bytes.as_ptr(),
                g2_bytes.len() / BYTES_PER_G2,
            )
        };
        assert_eq!(status, C_KZG_RET_OK);

        let mut rng = rand::thread_rng();
        let mut blob_bytes = generate_random_blob_bytes(&mut rng);

        let bls_modulus: [u8; BYTES_PER_FIELD_ELEMENT] = [
            0x73, 0xED, 0xA7, 0x53, 0x29, 0x9D, 0x7D, 0x48, 0x33, 0x39, 0xD8, 0x08, 0x09, 0xA1,
            0xD8, 0x05, 0x53, 0xBD, 0xA4, 0x02, 0xFF, 0xFE, 0x5B, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF,
            0x00, 0x00, 0x00, 0x01,
        ];
        // Make first field element equal to BLS_MODULUS
        blob_bytes[0..BYTES_PER_FIELD_ELEMENT].copy_from_slice(&bls_modulus);

        let blob = Blob { bytes: blob_bytes };
        let mut commitment = KZGCommitment {
            bytes: [0; BYTES_PER_COMMITMENT],
        };

        let output = unsafe { blob_to_kzg_commitment(&mut commitment, &blob, &c_settings) };

        assert_eq!(output, C_KZG_RET_BADARGS)
    }

    #[test]
    pub fn load_trusted_setup_invalid_g1_byte_length() {
        let mut file = File::open(get_trusted_setup_path()).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let (mut g1_bytes, g2_bytes) = load_trusted_setup_string(&contents).unwrap();
        // Add one more point
        let additional = [0; BYTES_PER_G1];
        g1_bytes.extend_from_slice(&additional);

        let mut loaded_settings = CKZGSettings {
            g1_values: null_mut(),
            g2_values: null_mut(),
            max_width: 0,
            roots_of_unity: null_mut(),
        };

        let status = unsafe {
            load_trusted_setup(
                &mut loaded_settings,
                g1_bytes.as_ptr(),
                g1_bytes.len() / BYTES_PER_G1,
                g2_bytes.as_ptr(),
                g2_bytes.len() / BYTES_PER_G2,
            )
        };

        assert_eq!(status, C_KZG_RET_BADARGS)
    }

    #[test]
    pub fn load_trusted_setup_invalid_g2_byte_length() {
        let mut file = File::open(get_trusted_setup_path()).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let (g1_bytes, mut g2_bytes) = load_trusted_setup_string(&contents).unwrap();
        // Add one more point
        let additional = [0; BYTES_PER_G2];
        g2_bytes.extend_from_slice(&additional);

        let mut loaded_settings = CKZGSettings {
            g1_values: null_mut(),
            g2_values: null_mut(),
            max_width: 0,
            roots_of_unity: null_mut(),
        };

        let status = unsafe {
            load_trusted_setup(
                &mut loaded_settings,
                g1_bytes.as_ptr(),
                g1_bytes.len() / BYTES_PER_G1,
                g2_bytes.as_ptr(),
                g2_bytes.len() / BYTES_PER_G2,
            )
        };

        assert_eq!(status, C_KZG_RET_BADARGS)
    }

    #[test]
    pub fn load_trusted_setup_invalid_g1_point() {
        let mut file = File::open(get_trusted_setup_path()).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let (mut g1_bytes, g2_bytes) = load_trusted_setup_string(&contents).unwrap();
        // Break first G1 point
        g1_bytes[0] = 0;

        let mut loaded_settings = CKZGSettings {
            g1_values: null_mut(),
            g2_values: null_mut(),
            max_width: 0,
            roots_of_unity: null_mut(),
        };

        let status = unsafe {
            load_trusted_setup(
                &mut loaded_settings,
                g1_bytes.as_ptr(),
                g1_bytes.len() / BYTES_PER_G1,
                g2_bytes.as_ptr(),
                g2_bytes.len() / BYTES_PER_G2,
            )
        };

        assert_eq!(status, C_KZG_RET_BADARGS)
    }

    #[test]
    pub fn load_trusted_setup_invalid_g2_point() {
        let mut file = File::open(get_trusted_setup_path()).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let (g1_bytes, mut g2_bytes) = load_trusted_setup_string(&contents).unwrap();
        // Break first G2 point
        g2_bytes[0] = 0;

        let mut loaded_settings = CKZGSettings {
            g1_values: null_mut(),
            g2_values: null_mut(),
            max_width: 0,
            roots_of_unity: null_mut(),
        };

        let status = unsafe {
            load_trusted_setup(
                &mut loaded_settings,
                g1_bytes.as_ptr(),
                g1_bytes.len() / BYTES_PER_G1,
                g2_bytes.as_ptr(),
                g2_bytes.len() / BYTES_PER_G2,
            )
        };

        assert_eq!(status, C_KZG_RET_BADARGS)
    }

    #[test]
    pub fn load_trusted_setup_invalid_form() {
        let trusted_setup_name = if cfg!(feature = "minimal-spec") {
            "trusted_setup_4_old.txt"
        } else {
            "trusted_setup_old.txt"
        };
        let mut file = File::open(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(trusted_setup_name)
                .as_os_str()
                .to_str()
                .unwrap(),
        )
        .unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let (g1_bytes, g2_bytes) = load_trusted_setup_string(&contents).unwrap();

        let mut loaded_settings = CKZGSettings {
            g1_values: null_mut(),
            g2_values: null_mut(),
            max_width: 0,
            roots_of_unity: null_mut(),
        };

        let status = unsafe {
            load_trusted_setup(
                &mut loaded_settings,
                g1_bytes.as_ptr(),
                g1_bytes.len() / BYTES_PER_G1,
                g2_bytes.as_ptr(),
                g2_bytes.len() / BYTES_PER_G2,
            )
        };

        assert_eq!(status, C_KZG_RET_BADARGS)
    }

    #[test]
    pub fn expand_root_of_unity_too_long() {
        let out = expand_root_of_unity(&FsFr::from_u64_arr(&SCALE2_ROOT_OF_UNITY[1]), 1);
        assert!(out.is_err());
    }

    #[test]
    pub fn expand_root_of_unity_too_short() {
        let out = expand_root_of_unity(&FsFr::from_u64_arr(&SCALE2_ROOT_OF_UNITY[1]), 1);
        assert!(out.is_err());
    }
}
