#[cfg(test)]

mod tests {
    // use blst_from_scratch::types::{fr::FsFr, g1::FsG1};
    // use kzg_bench::tests::eip_4844::{
    //     g1_lincomb
    // };

    // #[test]
    // pub fn test_g1_lincomb() {
    //     g1_lincomb_vienas_testas::<FsFr, FsG1>(
    //         &g1_lincomb
    //     );
    // }

    use blst_from_scratch::{
        eip_4844::{bytes_to_bls_field, compute_powers, load_trusted_setup, evaluate_polynomial_in_evaluation_form},
        types::{fr::FsFr, kzg_settings::FsKZGSettings, fft_settings::FsFFTSettings, poly::FsPoly, g2::FsG2, g1::FsG1},
    };
    use kzg_bench::tests::eip_4844::{bytes_to_bls_field_test, compute_powers_test, evaluate_polynomial_in_evaluation_form_test};

    #[test]
    pub fn bytes_to_bls_field_test_() {
        bytes_to_bls_field_test::<FsFr>(&bytes_to_bls_field);
    }

    #[test]
    pub fn compute_powers_test_() {
        compute_powers_test::<FsFr>(&compute_powers, &bytes_to_bls_field);
    }

    #[test]
    pub fn evaluate_polynomial_in_evaluation_form_test_() {
        evaluate_polynomial_in_evaluation_form_test::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(&evaluate_polynomial_in_evaluation_form, &bytes_to_bls_field, &load_trusted_setup);
    }
}
