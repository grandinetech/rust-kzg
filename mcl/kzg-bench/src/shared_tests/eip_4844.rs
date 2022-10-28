use mcl_rust::eip_4844::*;
use mcl_rust::CurveType;
use mcl_rust::mcl_methods::init;
use kzg_bench::tests::eip_4844::*;
use mcl_rust::data_types::{fr::Fr, g1::G1, g2::G2};
use mcl_rust::kzg_settings::KZGSettings;
use mcl_rust::fk20_fft::FFTSettings;
use mcl_rust::kzg10::Polynomial;

#[test]
pub fn test_bytes_to_bls_field() {
    assert!(init(CurveType::BLS12_381));
    bytes_to_bls_field_test(&bytes_to_bls_field, &bytes_from_bls_field);
}

#[test]
pub fn test_compute_powers() {
    compute_powers_test::<Fr>(&bytes_to_bls_field, &compute_powers);
}


#[test]
pub fn test_evaluate_polynomial_in_evaluation_form() {
    assert!(init(CurveType::BLS12_381));
    evaluate_polynomial_in_evaluation_form_test::<
            Fr,
            G1,
            G2,
            Polynomial,
            FFTSettings,
            KZGSettings,
        >(
            &bytes_to_bls_field,
            &load_trusted_setup,
            &evaluate_polynomial_in_evaluation_form,
        );
}
