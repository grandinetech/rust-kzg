#[cfg(test)]
mod tests {
    use kzg_bench::tests::eip_4844::*;
    use rust_kzg_mcl::data_types::{fr::Fr, g1::G1, g2::G2};
    use rust_kzg_mcl::eip_4844::*;
    use rust_kzg_mcl::fk20_fft::FFTSettings;
    use rust_kzg_mcl::kzg10::Polynomial;
    use rust_kzg_mcl::kzg_settings::KZGSettings;
    use rust_kzg_mcl::mcl_methods::init;
    use rust_kzg_mcl::CurveType;

    #[test]
    pub fn bytes_to_bls_field_test_() {
        assert!(init(CurveType::BLS12_381));
        bytes_to_bls_field_test::<Fr>();
    }

    #[test]
    pub fn compute_powers_test_() {
        assert!(init(CurveType::BLS12_381));
        compute_powers_test::<Fr>(&compute_powers);
    }

    #[test]
    pub fn blob_to_kzg_commitment_test_() {
        assert!(init(CurveType::BLS12_381));
        blob_to_kzg_commitment_test::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings>(
            &load_trusted_setup,
            &blob_to_kzg_commitment,
        );
    }

    #[test]
    pub fn compute_kzg_proof_test_() {
        compute_kzg_proof_test::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings>(
            &load_trusted_setup,
            &compute_kzg_proof,
            &blob_to_polynomial,
            &evaluate_polynomial_in_evaluation_form,
        );
    }

    #[test]
    pub fn compute_and_verify_kzg_proof_round_trip_test_() {
        compute_and_verify_kzg_proof_round_trip_test::<
            Fr,
            G1,
            G2,
            Polynomial,
            FFTSettings,
            KZGSettings,
        >(
            &load_trusted_setup,
            &blob_to_kzg_commitment,
            &bytes_to_blob,
            &compute_kzg_proof,
            &blob_to_polynomial,
            &evaluate_polynomial_in_evaluation_form,
            &verify_kzg_proof,
        );
    }

    //#[cfg(not(feature = "minimal-spec"))]
    //#[test]
    //pub fn compute_and_verify_kzg_proof_within_domain_test_() {
    //    compute_and_verify_kzg_proof_within_domain_test::<
    //        Fr,
    //        G1,
    //        G2,
    //        Polynomial,
    //        FFTSettings,
    //        KZGSettings,
    //    >(
    //        &load_trusted_setup,
    //        &blob_to_kzg_commitment,
    //        &bytes_to_blob,
    //        &compute_kzg_proof,
    //        &blob_to_polynomial,
    //        &evaluate_polynomial_in_evaluation_form,
    //        &verify_kzg_proof,
    //    );
    //}

    #[test]
    pub fn compute_and_verify_kzg_proof_fails_with_incorrect_proof_test_() {
        compute_and_verify_kzg_proof_fails_with_incorrect_proof_test::<
            Fr,
            G1,
            G2,
            Polynomial,
            FFTSettings,
            KZGSettings,
        >(
            &load_trusted_setup,
            &blob_to_kzg_commitment,
            &bytes_to_blob,
            &compute_kzg_proof,
            &blob_to_polynomial,
            &evaluate_polynomial_in_evaluation_form,
            &verify_kzg_proof,
        );
    }

    #[test]
    pub fn compute_and_verify_blob_kzg_proof_test_() {
        assert!(init(CurveType::BLS12_381));
        compute_and_verify_blob_kzg_proof_test::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings>(
            &load_trusted_setup,
            &blob_to_kzg_commitment,
            &bytes_to_blob,
            &compute_blob_kzg_proof,
            &verify_blob_kzg_proof,
        );
    }

    #[test]
    pub fn compute_and_verify_blob_kzg_proof_fails_with_incorrect_proof_test_() {
        assert!(init(CurveType::BLS12_381));
        compute_and_verify_blob_kzg_proof_fails_with_incorrect_proof_test::<
            Fr,
            G1,
            G2,
            Polynomial,
            FFTSettings,
            KZGSettings,
        >(
            &load_trusted_setup,
            &blob_to_kzg_commitment,
            &bytes_to_blob,
            &compute_blob_kzg_proof,
            &verify_blob_kzg_proof,
        );
    }

    #[test]
    pub fn verify_kzg_proof_batch_test_() {
        assert!(init(CurveType::BLS12_381));
        verify_kzg_proof_batch_test::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings>(
            &load_trusted_setup,
            &blob_to_kzg_commitment,
            &bytes_to_blob,
            &compute_blob_kzg_proof,
            &verify_blob_kzg_proof_batch,
        );
    }

    #[test]
    pub fn verify_kzg_proof_batch_fails_with_incorrect_proof_test_() {
        assert!(init(CurveType::BLS12_381));
        verify_kzg_proof_batch_fails_with_incorrect_proof_test::<
            Fr,
            G1,
            G2,
            Polynomial,
            FFTSettings,
            KZGSettings,
        >(
            &load_trusted_setup,
            &blob_to_kzg_commitment,
            &bytes_to_blob,
            &compute_blob_kzg_proof,
            &verify_blob_kzg_proof_batch,
        );
    }
}
