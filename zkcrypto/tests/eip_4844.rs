#[cfg(test)]

mod tests {
    use kzg_bench::tests::eip_4844::{
        bytes_to_bls_field_test, compute_commitment_for_blobs_test, compute_powers_test,
        evaluate_polynomial_in_evaluation_form_test,
    };
    use zkcrypto::eip_4844::{bytes_from_bls_field, bytes_to_bls_field, compute_powers};
    use zkcrypto::zkfr::blsScalar;

    #[test]
    pub fn bytes_to_bls_field_test_() {
        bytes_to_bls_field_test::<blsScalar>(&bytes_to_bls_field, &bytes_from_bls_field);
    }

    #[test]
    pub fn compute_powers_test_() {
        compute_powers_test::<blsScalar>(&bytes_to_bls_field, &compute_powers);
    }
}
