#[cfg(test)]
mod tests {
    use blst_from_scratch::eip_4844::{
        blob_to_polynomial_rust, bytes_to_bls_field_rust, bytes_to_g1_rust,
        compute_blob_kzg_proof_rust, compute_kzg_proof_rust,
        evaluate_polynomial_in_evaluation_form_rust, verify_blob_kzg_proof_batch_rust,
        verify_blob_kzg_proof_rust, verify_kzg_proof_rust,
    };
    use blst_from_scratch::{
        eip_4844::{
            blob_to_kzg_commitment_rust, bytes_from_bls_field, compute_powers, hash_to_bls_field,
            load_trusted_setup_filename_rust,
        },
        types::{
            fft_settings::FsFFTSettings, fr::FsFr, g1::FsG1, g2::FsG2, kzg_settings::FsKZGSettings,
            poly::FsPoly,
        },
    };
    use kzg_bench::tests::eip_4844::{
        blob_to_kzg_commitment_test, bytes_to_bls_field_test,
        compute_and_verify_blob_kzg_proof_fails_with_incorrect_proof_test,
        compute_and_verify_blob_kzg_proof_test,
        compute_and_verify_kzg_proof_fails_with_incorrect_proof_test,
        compute_and_verify_kzg_proof_round_trip_test,
        compute_and_verify_kzg_proof_within_domain_test, compute_kzg_proof_test,
        compute_powers_test, verify_kzg_proof_batch_fails_with_incorrect_proof_test,
        verify_kzg_proof_batch_test,
    };

    #[test]
    pub fn bytes_to_bls_field_test_() {
        bytes_to_bls_field_test::<FsFr>(&hash_to_bls_field, &bytes_from_bls_field);
    }

    #[test]
    pub fn compute_powers_test_() {
        compute_powers_test::<FsFr>(&hash_to_bls_field, &compute_powers);
    }

    #[test]
    pub fn blob_to_kzg_commitment_test_() {
        blob_to_kzg_commitment_test::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &load_trusted_setup_filename_rust,
            &blob_to_kzg_commitment_rust,
            &bytes_to_bls_field_rust,
            &bytes_to_g1_rust,
        );
    }

    #[test]
    pub fn compute_kzg_proof_test_() {
        compute_kzg_proof_test::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &load_trusted_setup_filename_rust,
            &bytes_to_bls_field_rust,
            &bytes_to_g1_rust,
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
            &bytes_to_bls_field_rust,
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
            &bytes_to_bls_field_rust,
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
            &bytes_to_bls_field_rust,
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
            &bytes_to_bls_field_rust,
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
            &bytes_to_bls_field_rust,
            &compute_blob_kzg_proof_rust,
            &verify_blob_kzg_proof_rust,
        );
    }

    #[test]
    pub fn verify_kzg_proof_batch_test_() {
        verify_kzg_proof_batch_test::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &load_trusted_setup_filename_rust,
            &blob_to_kzg_commitment_rust,
            &bytes_to_bls_field_rust,
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
            &bytes_to_bls_field_rust,
            &compute_blob_kzg_proof_rust,
            &verify_blob_kzg_proof_batch_rust,
        );
    }
}
