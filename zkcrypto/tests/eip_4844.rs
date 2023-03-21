#[cfg(test)]
mod tests {
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
    use zkcrypto::eip_4844::{
        blob_to_kzg_commitment, blob_to_polynomial, bytes_from_bls_field, bytes_to_bls_field,
        bytes_to_g1, compute_blob_kzg_proof, compute_kzg_proof, compute_powers,
        evaluate_polynomial_in_evaluation_form, hash_to_bls_field, load_trusted_setup,
        verify_blob_kzg_proof, verify_blob_kzg_proof_batch, verify_kzg_proof,
    };
    use zkcrypto::fftsettings::ZkFFTSettings;
    use zkcrypto::kzg_proofs::KZGSettings;
    use zkcrypto::kzg_types::ZkG2Projective;
    use zkcrypto::poly::KzgPoly;
    use zkcrypto::utils::ZkG1Projective;
    use zkcrypto::zkfr::blsScalar;

    #[test]
    pub fn bytes_to_bls_field_test_() {
        bytes_to_bls_field_test::<blsScalar>(&hash_to_bls_field, &bytes_from_bls_field);
    }

    #[test]
    pub fn compute_powers_test_() {
        compute_powers_test::<blsScalar>(&hash_to_bls_field, &compute_powers);
    }

    #[test]
    pub fn blob_to_kzg_commitment_test_() {
        blob_to_kzg_commitment_test::<
            blsScalar,
            ZkG1Projective,
            ZkG2Projective,
            KzgPoly,
            ZkFFTSettings,
            KZGSettings,
        >(
            &load_trusted_setup,
            &blob_to_kzg_commitment,
            &bytes_to_bls_field,
            &bytes_to_g1,
        );
    }

    #[test]
    pub fn compute_kzg_proof_test_() {
        compute_kzg_proof_test::<
            blsScalar,
            ZkG1Projective,
            ZkG2Projective,
            KzgPoly,
            ZkFFTSettings,
            KZGSettings,
        >(
            &load_trusted_setup,
            &bytes_to_bls_field,
            &bytes_to_g1,
            &compute_kzg_proof,
        );
    }

    #[test]
    pub fn compute_and_verify_kzg_proof_round_trip_test_() {
        compute_and_verify_kzg_proof_round_trip_test::<
            blsScalar,
            ZkG1Projective,
            ZkG2Projective,
            KzgPoly,
            ZkFFTSettings,
            KZGSettings,
        >(
            &load_trusted_setup,
            &blob_to_kzg_commitment,
            &bytes_to_bls_field,
            &compute_kzg_proof,
            &blob_to_polynomial,
            &evaluate_polynomial_in_evaluation_form,
            &verify_kzg_proof,
        );
    }

    #[test]
    pub fn compute_and_verify_kzg_proof_within_domain_test_() {
        compute_and_verify_kzg_proof_within_domain_test::<
            blsScalar,
            ZkG1Projective,
            ZkG2Projective,
            KzgPoly,
            ZkFFTSettings,
            KZGSettings,
        >(
            &load_trusted_setup,
            &blob_to_kzg_commitment,
            &bytes_to_bls_field,
            &compute_kzg_proof,
            &blob_to_polynomial,
            &evaluate_polynomial_in_evaluation_form,
            &verify_kzg_proof,
        );
    }

    #[test]
    pub fn compute_and_verify_kzg_proof_fails_with_incorrect_proof_test_() {
        compute_and_verify_kzg_proof_fails_with_incorrect_proof_test::<
            blsScalar,
            ZkG1Projective,
            ZkG2Projective,
            KzgPoly,
            ZkFFTSettings,
            KZGSettings,
        >(
            &load_trusted_setup,
            &blob_to_kzg_commitment,
            &bytes_to_bls_field,
            &compute_kzg_proof,
            &blob_to_polynomial,
            &evaluate_polynomial_in_evaluation_form,
            &verify_kzg_proof,
        );
    }

    #[test]
    pub fn compute_and_verify_blob_kzg_proof_test_() {
        compute_and_verify_blob_kzg_proof_test::<
            blsScalar,
            ZkG1Projective,
            ZkG2Projective,
            KzgPoly,
            ZkFFTSettings,
            KZGSettings,
        >(
            &load_trusted_setup,
            &blob_to_kzg_commitment,
            &bytes_to_bls_field,
            &compute_blob_kzg_proof,
            &verify_blob_kzg_proof,
        );
    }

    #[test]
    pub fn compute_and_verify_blob_kzg_proof_fails_with_incorrect_proof_test_() {
        compute_and_verify_blob_kzg_proof_fails_with_incorrect_proof_test::<
            blsScalar,
            ZkG1Projective,
            ZkG2Projective,
            KzgPoly,
            ZkFFTSettings,
            KZGSettings,
        >(
            &load_trusted_setup,
            &blob_to_kzg_commitment,
            &bytes_to_bls_field,
            &compute_blob_kzg_proof,
            &verify_blob_kzg_proof,
        );
    }

    #[test]
    pub fn verify_kzg_proof_batch_test_() {
        verify_kzg_proof_batch_test::<
            blsScalar,
            ZkG1Projective,
            ZkG2Projective,
            KzgPoly,
            ZkFFTSettings,
            KZGSettings,
        >(
            &load_trusted_setup,
            &blob_to_kzg_commitment,
            &bytes_to_bls_field,
            &compute_blob_kzg_proof,
            &verify_blob_kzg_proof_batch,
        );
    }

    #[test]
    pub fn verify_kzg_proof_batch_fails_with_incorrect_proof_test_() {
        verify_kzg_proof_batch_fails_with_incorrect_proof_test::<
            blsScalar,
            ZkG1Projective,
            ZkG2Projective,
            KzgPoly,
            ZkFFTSettings,
            KZGSettings,
        >(
            &load_trusted_setup,
            &blob_to_kzg_commitment,
            &bytes_to_bls_field,
            &compute_blob_kzg_proof,
            &verify_blob_kzg_proof_batch,
        );
    }
}
