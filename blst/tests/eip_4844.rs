#[cfg(test)]
mod tests {

    use kzg::Fr;

    use kzg_bench::tests::eip_4844::{
        blob_to_kzg_commitment_test, bytes_to_bls_field_test,
        compute_and_verify_blob_kzg_proof_fails_with_incorrect_proof_test,
        compute_and_verify_blob_kzg_proof_test,
        compute_and_verify_kzg_proof_fails_with_incorrect_proof_test,
        compute_and_verify_kzg_proof_round_trip_test,
        compute_and_verify_kzg_proof_within_domain_test, compute_kzg_proof_empty_blob_vector_test,
        compute_kzg_proof_incorrect_blob_length_test,
        compute_kzg_proof_incorrect_commitments_len_test,
        compute_kzg_proof_incorrect_poly_length_test, compute_kzg_proof_incorrect_proofs_len_test,
        compute_kzg_proof_test, compute_powers_test, test_vectors_blob_to_kzg_commitment,
        test_vectors_compute_blob_kzg_proof, test_vectors_compute_kzg_proof,
        test_vectors_verify_blob_kzg_proof, test_vectors_verify_blob_kzg_proof_batch,
        test_vectors_verify_kzg_proof, validate_batched_input_test,
        verify_kzg_proof_batch_fails_with_incorrect_proof_test, verify_kzg_proof_batch_test,
    };
    use rust_kzg_blst::consts::SCALE2_ROOT_OF_UNITY;
    use rust_kzg_blst::eip_4844::{
        blob_to_kzg_commitment_rust, blob_to_polynomial_rust, bytes_to_blob,
        compute_blob_kzg_proof_rust, compute_kzg_proof_rust, compute_powers,
        evaluate_polynomial_in_evaluation_form_rust, load_trusted_setup_filename_rust,
        verify_blob_kzg_proof_batch_rust, verify_blob_kzg_proof_rust, verify_kzg_proof_rust,
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

    #[test]
    pub fn test_vectors_blob_to_kzg_commitment_() {
        test_vectors_blob_to_kzg_commitment::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &load_trusted_setup_filename_rust,
            &blob_to_kzg_commitment_rust,
            &bytes_to_blob,
        );
    }

    #[test]
    pub fn test_vectors_compute_kzg_proof_() {
        test_vectors_compute_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &load_trusted_setup_filename_rust,
            &compute_kzg_proof_rust,
            &bytes_to_blob,
        );
    }

    #[test]
    pub fn test_vectors_compute_blob_kzg_proof_() {
        test_vectors_compute_blob_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &load_trusted_setup_filename_rust,
            &bytes_to_blob,
            &compute_blob_kzg_proof_rust,
        );
    }

    #[test]
    pub fn test_vectors_verify_kzg_proof_() {
        test_vectors_verify_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &load_trusted_setup_filename_rust,
            &verify_kzg_proof_rust,
        );
    }

    #[test]
    pub fn test_vectors_verify_blob_kzg_proof_() {
        test_vectors_verify_blob_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &load_trusted_setup_filename_rust,
            &bytes_to_blob,
            &verify_blob_kzg_proof_rust,
        );
    }

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
    pub fn expand_root_of_unity_too_long() {
        let out = expand_root_of_unity(&FsFr::from_u64_arr(&SCALE2_ROOT_OF_UNITY[1]), 1);
        assert!(out.is_err());
    }

    #[test]
    pub fn expand_root_of_unity_too_short() {
        let out = expand_root_of_unity(&FsFr::from_u64_arr(&SCALE2_ROOT_OF_UNITY[1]), 3);
        assert!(out.is_err());
    }

    #[test]
    pub fn compute_kzg_proof_incorrect_blob_length() {
        compute_kzg_proof_incorrect_blob_length_test::<FsFr, FsPoly>(&blob_to_polynomial_rust);
    }

    #[test]
    pub fn compute_kzg_proof_incorrect_poly_length() {
        compute_kzg_proof_incorrect_poly_length_test::<
            FsPoly,
            FsFr,
            FsG1,
            FsG2,
            FsFFTSettings,
            FsKZGSettings,
        >(&evaluate_polynomial_in_evaluation_form_rust);
    }

    #[test]
    pub fn compute_kzg_proof_empty_blob_vector() {
        compute_kzg_proof_empty_blob_vector_test::<
            FsPoly,
            FsFr,
            FsG1,
            FsG2,
            FsFFTSettings,
            FsKZGSettings,
        >(&verify_blob_kzg_proof_batch_rust)
    }

    #[test]
    pub fn compute_kzg_proof_incorrect_commitments_len() {
        compute_kzg_proof_incorrect_commitments_len_test::<
            FsPoly,
            FsFr,
            FsG1,
            FsG2,
            FsFFTSettings,
            FsKZGSettings,
        >(&verify_blob_kzg_proof_batch_rust)
    }

    #[test]
    pub fn compute_kzg_proof_incorrect_proofs_len() {
        compute_kzg_proof_incorrect_proofs_len_test::<
            FsPoly,
            FsFr,
            FsG1,
            FsG2,
            FsFFTSettings,
            FsKZGSettings,
        >(&verify_blob_kzg_proof_batch_rust)
    }

    #[test]
    pub fn validate_batched_input() {
        validate_batched_input_test::<FsPoly, FsFr, FsG1, FsG2, FsFFTSettings, FsKZGSettings>(
            &verify_blob_kzg_proof_batch_rust,
            &load_trusted_setup_filename_rust,
        )
    }
}
