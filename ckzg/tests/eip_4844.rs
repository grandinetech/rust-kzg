#[cfg(test)]

mod tests {

    use ckzg::{
        consts::{BlstP1, BlstP2},
        eip_4844::{
            blob_to_kzg_commitment, bytes_from_bls_field, bytes_from_g1, bound_bytes_to_bls_field,
            compute_kzg_proof, compute_powers, evaluate_polynomial_in_evaluation_form, g1_lincomb,
            load_trusted_setup, vector_lincomb, verify_kzg_proof,
        },
        fftsettings::KzgFFTSettings,
        finite::BlstFr,
        kzgsettings::KzgKZGSettings,
        poly::KzgPoly,
    };
    use kzg_bench::tests::eip_4844::{
        bytes_to_bls_field_test, compute_commitment_for_blobs_test, compute_powers_test,
        evaluate_polynomial_in_evaluation_form_test,
    };

    #[test]
    pub fn bytes_to_bls_field_test_() {
        bytes_to_bls_field_test::<BlstFr>(&bound_bytes_to_bls_field, &bytes_from_bls_field);
    }

    #[test]
    pub fn compute_powers_test_() {
        compute_powers_test::<BlstFr>(&bound_bytes_to_bls_field, &compute_powers);
    }

    #[test]
    pub fn evaluate_polynomial_in_evaluation_form_test_() {
        evaluate_polynomial_in_evaluation_form_test::<
            BlstFr,
            BlstP1,
            BlstP2,
            KzgPoly,
            KzgFFTSettings,
            KzgKZGSettings,
        >(
            &bound_bytes_to_bls_field,
            &load_trusted_setup,
            &evaluate_polynomial_in_evaluation_form,
        );
    }

    #[test]
    pub fn compute_commitment_for_blobs_test_() {
        compute_commitment_for_blobs_test::<
            BlstFr,
            BlstP1,
            BlstP2,
            KzgPoly,
            KzgFFTSettings,
            KzgKZGSettings,
        >(
            &load_trusted_setup,
            &bound_bytes_to_bls_field,
            &bytes_from_bls_field,
            &bytes_from_g1,
            &compute_powers,
            &vector_lincomb,
            &g1_lincomb,
            &evaluate_polynomial_in_evaluation_form,
            &blob_to_kzg_commitment,
            &compute_kzg_proof,
            &verify_kzg_proof,
        )
    }
}
