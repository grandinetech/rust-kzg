#[cfg(test)]

mod tests {
    use kzg_bench::tests::eip_4844::{
        bytes_to_bls_field_test, compute_powers_test, evaluate_polynomial_in_evaluation_form_test
    };
    use zkcrypto::eip_4844::{
        bytes_from_bls_field, bytes_to_bls_field, compute_powers, load_trusted_setup,
        evaluate_polynomial_in_evaluation_form
    };
    use zkcrypto::fftsettings::ZkFFTSettings;
    use zkcrypto::kzg_proofs::KZGSettings;
    use zkcrypto::kzg_types::{ZkG1Projective, ZkG2Projective};
    use zkcrypto::zkfr::blsScalar;
    use zkcrypto::ZPoly;

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
            ZPoly,
            ZkFFTSettings,
            KZGSettings,
        >(
            &bytes_to_bls_field,
            &load_trusted_setup,
            &evaluate_polynomial_in_evaluation_form,
        );
    }
}
