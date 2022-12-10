#[cfg(test)]
pub mod tests {
    use arkworks::eip_4844::{bytes_from_bls_field, bytes_to_bls_field, compute_powers, evaluate_polynomial_in_evaluation_form, load_trusted_setup};
    use kzg_bench::tests::eip_4844::{bytes_to_bls_field_test, compute_powers_test, evaluate_polynomial_in_evaluation_form_test};
    use arkworks::kzg_proofs::{FFTSettings, KZGSettings};
    use arkworks::kzg_types::{ArkG1, ArkG2, FsFr};
    use arkworks::utils::PolyData;

    #[test]
    fn bytes_to_bls_field_test_() {
        bytes_to_bls_field_test::<FsFr>(&bytes_to_bls_field, &bytes_from_bls_field);
    }

    #[test]
    pub fn compute_powers_test_() {
        compute_powers_test::<FsFr>(&bytes_to_bls_field, &compute_powers);
    }

    #[test]
    pub fn evaluate_polynomial_in_evaluation_form_test_() {
        evaluate_polynomial_in_evaluation_form_test::<
            FsFr,
            ArkG1,
            ArkG2,
            PolyData,
            FFTSettings,
            KZGSettings,
        >(
            &bytes_to_bls_field,
            &load_trusted_setup,
            &evaluate_polynomial_in_evaluation_form,
        );
    }
}
