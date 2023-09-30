#[cfg(test)]
mod tests {
    use std::env::set_current_dir;

    use kzg::eip_4844::{
        Blob, KZGCommitment, BYTES_PER_COMMITMENT, BYTES_PER_FIELD_ELEMENT, C_KZG_RET_BADARGS,
        TRUSTED_SETUP_PATH,
    };
    use kzg_bench::tests::eip_4844::{
        blob_to_kzg_commitment_test, bytes_to_bls_field_test,
        compute_and_verify_blob_kzg_proof_fails_with_incorrect_proof_test,
        compute_and_verify_blob_kzg_proof_test,
        compute_and_verify_kzg_proof_fails_with_incorrect_proof_test,
        compute_and_verify_kzg_proof_round_trip_test, compute_kzg_proof_test, compute_powers_test,
        generate_random_blob_bytes, verify_kzg_proof_batch_fails_with_incorrect_proof_test,
        verify_kzg_proof_batch_test,
    };
    #[cfg(not(feature = "minimal-spec"))]
    use kzg_bench::tests::eip_4844::{
        compute_and_verify_kzg_proof_within_domain_test, test_vectors_blob_to_kzg_commitment,
        test_vectors_compute_blob_kzg_proof, test_vectors_compute_kzg_proof,
        test_vectors_verify_blob_kzg_proof, test_vectors_verify_blob_kzg_proof_batch,
        test_vectors_verify_kzg_proof,
    };
    use rust_kzg_blst::eip_4844::{
        blob_to_kzg_commitment, blob_to_kzg_commitment_rust, blob_to_polynomial_rust,
        bytes_to_blob, compute_blob_kzg_proof_rust, compute_kzg_proof_rust, compute_powers,
        evaluate_polynomial_in_evaluation_form_rust, kzg_settings_to_c,
        load_trusted_setup_filename_rust, verify_blob_kzg_proof_batch_rust,
        verify_blob_kzg_proof_rust, verify_kzg_proof_rust,
    };
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
        set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
        println!("{}", env!("CARGO_MANIFEST_DIR"));
        println!("{}", format!("../kzg-bench/{}", TRUSTED_SETUP_PATH));
        let settings =
            load_trusted_setup_filename_rust(&format!("../kzg-bench/{}", TRUSTED_SETUP_PATH))
                .unwrap();

        let c_settings = kzg_settings_to_c(&settings);

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
}
