#[cfg(test)]
mod tests {
    use kzg_bench::tests::eip_4844::{bytes_to_bls_field_test, compute_commitment_for_blobs_test, compute_powers_test, evaluate_polynomial_in_evaluation_form_test, eip4844_test, blob_to_kzg_commitment_test, compute_aggregate_kzg_proof_test_empty, aggregate_proof_for_single_blob_test};
    use zkcrypto::eip_4844::{
        bytes_from_bls_field, bytes_to_bls_field, compute_powers, evaluate_polynomial_in_evaluation_form,
        load_trusted_setup, bytes_from_g1, vector_lincomb, g1_lincomb,  blob_to_kzg_commitment,
        compute_kzg_proof, verify_kzg_proof, compute_aggregate_kzg_proof, verify_aggregate_kzg_proof
    };
    use zkcrypto::fftsettings::ZkFFTSettings;
    use zkcrypto::kzg_proofs::KZGSettings;
    use zkcrypto::kzg_types::ZkG2Projective;
    use zkcrypto::poly::KzgPoly;
    use zkcrypto::utils::ZkG1Projective;
    use zkcrypto::zkfr::blsScalar;

    #[test]
    pub fn bytes_to_bls_field_test_() {
        bytes_to_bls_field_test::<blsScalar>(&bytes_to_bls_field, &bytes_from_bls_field);
    }

    #[test]
    pub fn compute_powers_test_() {
        compute_powers_test::<blsScalar>(&bytes_to_bls_field, &compute_powers);
    }

    #[test]
    pub fn evaluate_polynomial_in_evaluation_form_test_() {
        evaluate_polynomial_in_evaluation_form_test::<
            blsScalar,
            ZkG1Projective,
            ZkG2Projective,
            KzgPoly,
            ZkFFTSettings,
            KZGSettings,
        >(
            &bytes_to_bls_field,
            &load_trusted_setup,
            &evaluate_polynomial_in_evaluation_form,
        );
    }

    #[test]
    pub fn compute_commitment_for_blobs_test_() {
        compute_commitment_for_blobs_test::<blsScalar, ZkG1Projective, ZkG2Projective, KzgPoly, ZkFFTSettings, KZGSettings>(
            &load_trusted_setup,
            &bytes_to_bls_field,
            &bytes_from_bls_field,
            &bytes_from_g1,
            &compute_powers,
            &vector_lincomb,
            &g1_lincomb,
            &evaluate_polynomial_in_evaluation_form,
            &blob_to_kzg_commitment,
            &compute_kzg_proof,
            &verify_kzg_proof,
        );
    }

    #[test]
    pub fn eip4844_test_() {
        eip4844_test::<blsScalar, ZkG1Projective, ZkG2Projective, KzgPoly, ZkFFTSettings, KZGSettings>(
            &load_trusted_setup,
            &blob_to_kzg_commitment,
            &compute_aggregate_kzg_proof,
            &verify_aggregate_kzg_proof,
        );
    }

    #[test]
    pub fn blob_to_kzg_commitment_test_() {
        blob_to_kzg_commitment_test::<blsScalar, ZkG1Projective, ZkG2Projective, KzgPoly, ZkFFTSettings, KZGSettings>(
            &load_trusted_setup,
            &blob_to_kzg_commitment,
            &bytes_from_g1,
        )
    }

    #[test]
    pub fn compute_aggregate_kzg_proof_test_empty_() {
        compute_aggregate_kzg_proof_test_empty::<blsScalar, ZkG1Projective, ZkG2Projective, KzgPoly, ZkFFTSettings, KZGSettings>(
            &load_trusted_setup,
            &compute_aggregate_kzg_proof,
            &bytes_from_g1,
        )
    }

    #[test]
    pub fn aggregate_proof_for_single_blob_test_() {
        aggregate_proof_for_single_blob_test::<blsScalar, ZkG1Projective, ZkG2Projective, KzgPoly, ZkFFTSettings, KZGSettings>(
            &load_trusted_setup,
            &blob_to_kzg_commitment,
            &compute_aggregate_kzg_proof,
            &verify_aggregate_kzg_proof,
        );
    }
}
