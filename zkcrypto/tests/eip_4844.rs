#[cfg(test)]
mod tests {
    use kzg::eip_4844::{
        blob_to_kzg_commitment_rust, blob_to_polynomial, bytes_to_blob,
        compute_blob_kzg_proof_rust, compute_kzg_proof_rust, compute_powers,
        evaluate_polynomial_in_evaluation_form, verify_blob_kzg_proof_batch_rust,
        verify_blob_kzg_proof_rust, verify_kzg_proof_rust,
    };
    use kzg::Fr;
    use kzg_bench::tests::eip_4844::{
        blob_to_kzg_commitment_test, bytes_to_bls_field_test,
        compute_and_verify_blob_kzg_proof_fails_with_incorrect_proof_test,
        compute_and_verify_blob_kzg_proof_test,
        compute_and_verify_kzg_proof_fails_with_incorrect_proof_test,
        compute_and_verify_kzg_proof_round_trip_test,
        compute_and_verify_kzg_proof_within_domain_test, compute_kzg_proof_test,
        compute_powers_test, test_vectors_blob_to_kzg_commitment,
        test_vectors_compute_blob_kzg_proof, test_vectors_compute_kzg_proof,
        test_vectors_verify_blob_kzg_proof, test_vectors_verify_blob_kzg_proof_batch,
        test_vectors_verify_kzg_proof, verify_kzg_proof_batch_fails_with_incorrect_proof_test,
        verify_kzg_proof_batch_test,
    };
    use rust_kzg_zkcrypto::consts::SCALE2_ROOT_OF_UNITY;
    use rust_kzg_zkcrypto::eip_4844::load_trusted_setup_filename_rust;
    use rust_kzg_zkcrypto::kzg_proofs::{expand_root_of_unity, FFTSettings, KZGSettings};
    use rust_kzg_zkcrypto::kzg_types::{ZFp, ZFr, ZG1Affine, ZG1ProjAddAffine, ZG1, ZG2};
    use rust_kzg_zkcrypto::poly::PolyData;

    #[test]
    pub fn bytes_to_bls_field_test_() {
        bytes_to_bls_field_test::<ZFr>();
    }

    #[test]
    pub fn compute_powers_test_() {
        compute_powers_test::<ZFr>(&compute_powers);
    }

    #[test]
    pub fn blob_to_kzg_commitment_test_() {
        blob_to_kzg_commitment_test::<
            ZFr,
            ZG1,
            ZG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            ZFp,
            ZG1Affine,
            ZG1ProjAddAffine,
        >(
            &load_trusted_setup_filename_rust,
            &blob_to_kzg_commitment_rust,
        );
    }

    #[test]
    pub fn compute_kzg_proof_test_() {
        compute_kzg_proof_test::<
            ZFr,
            ZG1,
            ZG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            ZFp,
            ZG1Affine,
            ZG1ProjAddAffine,
        >(
            &load_trusted_setup_filename_rust,
            &compute_kzg_proof_rust,
            &blob_to_polynomial,
            &evaluate_polynomial_in_evaluation_form,
        );
    }

    #[test]
    pub fn compute_and_verify_kzg_proof_round_trip_test_() {
        compute_and_verify_kzg_proof_round_trip_test::<
            ZFr,
            ZG1,
            ZG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            ZFp,
            ZG1Affine,
            ZG1ProjAddAffine,
        >(
            &load_trusted_setup_filename_rust,
            &blob_to_kzg_commitment_rust,
            &bytes_to_blob,
            &compute_kzg_proof_rust,
            &blob_to_polynomial,
            &evaluate_polynomial_in_evaluation_form,
            &verify_kzg_proof_rust,
        );
    }

    #[test]
    pub fn compute_and_verify_kzg_proof_within_domain_test_() {
        compute_and_verify_kzg_proof_within_domain_test::<
            ZFr,
            ZG1,
            ZG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            ZFp,
            ZG1Affine,
            ZG1ProjAddAffine,
        >(
            &load_trusted_setup_filename_rust,
            &blob_to_kzg_commitment_rust,
            &bytes_to_blob,
            &compute_kzg_proof_rust,
            &blob_to_polynomial,
            &evaluate_polynomial_in_evaluation_form,
            &verify_kzg_proof_rust,
        );
    }

    #[test]
    pub fn compute_and_verify_kzg_proof_fails_with_incorrect_proof_test_() {
        compute_and_verify_kzg_proof_fails_with_incorrect_proof_test::<
            ZFr,
            ZG1,
            ZG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            ZFp,
            ZG1Affine,
            ZG1ProjAddAffine,
        >(
            &load_trusted_setup_filename_rust,
            &blob_to_kzg_commitment_rust,
            &bytes_to_blob,
            &compute_kzg_proof_rust,
            &blob_to_polynomial,
            &evaluate_polynomial_in_evaluation_form,
            &verify_kzg_proof_rust,
        );
    }

    #[test]
    pub fn compute_and_verify_blob_kzg_proof_test_() {
        compute_and_verify_blob_kzg_proof_test::<
            ZFr,
            ZG1,
            ZG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            ZFp,
            ZG1Affine,
            ZG1ProjAddAffine,
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
            ZFr,
            ZG1,
            ZG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            ZFp,
            ZG1Affine,
            ZG1ProjAddAffine,
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
        verify_kzg_proof_batch_test::<
            ZFr,
            ZG1,
            ZG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            ZFp,
            ZG1Affine,
            ZG1ProjAddAffine,
        >(
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
            ZFr,
            ZG1,
            ZG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            ZFp,
            ZG1Affine,
            ZG1ProjAddAffine,
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
        test_vectors_blob_to_kzg_commitment::<
            ZFr,
            ZG1,
            ZG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            ZFp,
            ZG1Affine,
            ZG1ProjAddAffine,
        >(
            &load_trusted_setup_filename_rust,
            &blob_to_kzg_commitment_rust,
            &bytes_to_blob,
        );
    }

    #[test]
    pub fn test_vectors_compute_kzg_proof_() {
        test_vectors_compute_kzg_proof::<
            ZFr,
            ZG1,
            ZG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            ZFp,
            ZG1Affine,
            ZG1ProjAddAffine,
        >(
            &load_trusted_setup_filename_rust,
            &compute_kzg_proof_rust,
            &bytes_to_blob,
        );
    }

    #[test]
    pub fn test_vectors_compute_blob_kzg_proof_() {
        test_vectors_compute_blob_kzg_proof::<
            ZFr,
            ZG1,
            ZG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            ZFp,
            ZG1Affine,
            ZG1ProjAddAffine,
        >(
            &load_trusted_setup_filename_rust,
            &bytes_to_blob,
            &compute_blob_kzg_proof_rust,
        );
    }

    #[test]
    pub fn test_vectors_verify_kzg_proof_() {
        test_vectors_verify_kzg_proof::<
            ZFr,
            ZG1,
            ZG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            ZFp,
            ZG1Affine,
            ZG1ProjAddAffine,
        >(&load_trusted_setup_filename_rust, &verify_kzg_proof_rust);
    }

    #[test]
    pub fn test_vectors_verify_blob_kzg_proof_() {
        test_vectors_verify_blob_kzg_proof::<
            ZFr,
            ZG1,
            ZG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            ZFp,
            ZG1Affine,
            ZG1ProjAddAffine,
        >(
            &load_trusted_setup_filename_rust,
            &bytes_to_blob,
            &verify_blob_kzg_proof_rust,
        );
    }
    #[test]
    pub fn test_vectors_verify_blob_kzg_proof_batch_() {
        test_vectors_verify_blob_kzg_proof_batch::<
            ZFr,
            ZG1,
            ZG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            ZFp,
            ZG1Affine,
            ZG1ProjAddAffine,
        >(
            &load_trusted_setup_filename_rust,
            &bytes_to_blob,
            &verify_blob_kzg_proof_batch_rust,
        );
    }

    #[test]
    pub fn expand_root_of_unity_too_long() {
        let out = expand_root_of_unity(&ZFr::from_u64_arr(&SCALE2_ROOT_OF_UNITY[1]), 1);
        assert!(out.is_err());
    }

    #[test]
    pub fn expand_root_of_unity_too_short() {
        let out = expand_root_of_unity(&ZFr::from_u64_arr(&SCALE2_ROOT_OF_UNITY[1]), 3);
        assert!(out.is_err());
    }
}
