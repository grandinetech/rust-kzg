#[cfg(test)]

mod tests {

    use ckzg::{
        eip_4844::{bytes_to_bls_field, compute_powers, load_trusted_setup, evaluate_polynomial_in_evaluation_form},
        finite::BlstFr, kzgsettings::KzgKZGSettings, fftsettings::KzgFFTSettings, poly::KzgPoly, consts::{BlstP2, BlstP1},
    };
    use kzg_bench::tests::eip_4844::{bytes_to_bls_field_test, compute_powers_test, evaluate_polynomial_in_evaluation_form_test};

    #[test]
    pub fn bytes_to_bls_field_test_() {
        bytes_to_bls_field_test::<BlstFr>(&bytes_to_bls_field);
    }

    #[test]
    pub fn compute_powers_test_() {
        compute_powers_test::<BlstFr>(&bytes_to_bls_field, &compute_powers);
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
            &bytes_to_bls_field,
            &load_trusted_setup,
            &evaluate_polynomial_in_evaluation_form,
        );
    }
}
