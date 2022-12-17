#[cfg(test)]

mod tests {
    use blst_from_scratch::{
        eip_4844::{
            blob_to_kzg_commitment_rust, bytes_from_bls_field, bytes_from_g1_rust, bytes_to_bls_field_rust,
            compute_aggregate_kzg_proof_rust, compute_kzg_proof, compute_powers,
            evaluate_polynomial_in_evaluation_form, g1_lincomb, load_trusted_setup_rust, vector_lincomb,
            verify_aggregate_kzg_proof_rust, verify_kzg_proof_rust,
        },
        types::{
            fft_settings::FsFFTSettings, fr::FsFr, g1::FsG1, g2::FsG2, kzg_settings::FsKZGSettings,
            poly::FsPoly,
        },
    };
    use kzg_bench::tests::eip_4844::{
        blob_to_kzg_commitment_test, bytes_to_bls_field_test, compute_commitment_for_blobs_test,
        compute_powers_test, eip4844_test, evaluate_polynomial_in_evaluation_form_test, compute_aggregate_kzg_proof_test_empty, aggregate_proof_for_single_blob_test,
    };

    #[test]
    pub fn bytes_to_bls_field_test_() {
        bytes_to_bls_field_test::<FsFr>(&bytes_to_bls_field_rust, &bytes_from_bls_field);
    }

    #[test]
    pub fn compute_powers_test_() {
        compute_powers_test::<FsFr>(&bytes_to_bls_field_rust, &compute_powers);
    }

    #[test]
    pub fn evaluate_polynomial_in_evaluation_form_test_() {
        evaluate_polynomial_in_evaluation_form_test::<
            FsFr,
            FsG1,
            FsG2,
            FsPoly,
            FsFFTSettings,
            FsKZGSettings,
        >(
            &bytes_to_bls_field_rust,
            &load_trusted_setup_rust,
            &evaluate_polynomial_in_evaluation_form,
        );
    }

    #[test]
    pub fn compute_commitment_for_blobs_test_() {
        compute_commitment_for_blobs_test::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &load_trusted_setup_rust,
            &bytes_to_bls_field_rust,
            &bytes_from_bls_field,
            &bytes_from_g1_rust,
            &compute_powers,
            &vector_lincomb,
            &g1_lincomb,
            &evaluate_polynomial_in_evaluation_form,
            &blob_to_kzg_commitment_rust,
            &compute_kzg_proof,
            &verify_kzg_proof_rust,
        );
    }

    #[test]
    pub fn eip4844_test_() {
        eip4844_test::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &load_trusted_setup_rust,
            &blob_to_kzg_commitment_rust,
            &compute_aggregate_kzg_proof_rust,
            &verify_aggregate_kzg_proof_rust,
        );
    }

    #[test]
    pub fn blob_to_kzg_commitment_test_() {
        blob_to_kzg_commitment_test::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &load_trusted_setup_rust,
            &blob_to_kzg_commitment_rust,
            &bytes_from_g1_rust,
        )
    }

    #[test]
    pub fn compute_aggregate_kzg_proof_test_empty_() {
        compute_aggregate_kzg_proof_test_empty::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &load_trusted_setup_rust,
            &compute_aggregate_kzg_proof_rust,
            &bytes_from_g1_rust,
        )
    }

    // #[test]
    // pub fn verify_aggregate_kzg_proof_test_empty_() {
    //     verify_aggregate_kzg_proof_test_empty::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
    //         &load_trusted_setup,
    //         &compute_aggregate_kzg_proof,
    //         &verify_aggregate_kzg_proof,
    //     )
    // }

    #[test]
    pub fn aggregate_proof_for_single_blob_test_() {
        aggregate_proof_for_single_blob_test::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &load_trusted_setup_rust,
            &blob_to_kzg_commitment_rust,
            &compute_aggregate_kzg_proof_rust,
            &verify_aggregate_kzg_proof_rust,
        );
    }
}
