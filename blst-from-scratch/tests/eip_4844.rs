#[cfg(test)]

mod tests {
    use blst_from_scratch::{
        eip_4844::{
            blob_to_kzg_commitment_rust, bytes_from_bls_field, bytes_from_g1_rust, compute_powers,
            evaluate_polynomial_in_evaluation_form_rust, hash_to_bls_field,
            load_trusted_setup_filename_rust,
        },
        types::{
            fft_settings::FsFFTSettings, fr::FsFr, g1::FsG1, g2::FsG2, kzg_settings::FsKZGSettings,
            poly::FsPoly,
        },
    };
    use kzg_bench::tests::eip_4844::{
        blob_to_kzg_commitment_test, bytes_to_bls_field_test, compute_powers_test,
        evaluate_polynomial_in_evaluation_form_test,
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
    pub fn evaluate_polynomial_in_evaluation_form_test_() {
        evaluate_polynomial_in_evaluation_form_test::<
            FsFr,
            FsG1,
            FsG2,
            FsPoly,
            FsFFTSettings,
            FsKZGSettings,
        >(
            &hash_to_bls_field,
            &load_trusted_setup_filename_rust,
            &evaluate_polynomial_in_evaluation_form_rust,
        );
    }

    #[test]
    pub fn blob_to_kzg_commitment_test_() {
        blob_to_kzg_commitment_test::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &load_trusted_setup_filename_rust,
            &blob_to_kzg_commitment_rust,
            &bytes_from_g1_rust,
        )
    }
}
