#[cfg(test)]

mod tests {
    use zkcrypto::zkfr::blsScalar;
    use zkcrypto::{
        eip_4844::{
            bytes_to_bls_field, bytes_from_bls_field,
        },
    };
    use kzg_bench::tests::eip_4844::{
        bytes_to_bls_field_test, compute_commitment_for_blobs_test, compute_powers_test,
        evaluate_polynomial_in_evaluation_form_test,
    };

    #[test]
    pub fn bytes_to_bls_field_test_() {
        bytes_to_bls_field_test::<blsScalar>(&bytes_to_bls_field, &bytes_from_bls_field);
    }
}
