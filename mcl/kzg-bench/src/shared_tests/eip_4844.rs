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
    assert!(init(CurveType::BLS12_381));
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

#[test]
pub fn test_compute_commitment_for_blobs_test() {
    assert!(init(CurveType::BLS12_381));

    compute_commitment_for_blobs_test::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings>(
        &load_trusted_setup,
        &bytes_to_bls_field,
        &bytes_from_bls_field,
        &bytes_from_g1,
        &compute_powers,
        &vector_lincomb,
        &g1_lincomb,
        &evaluate_polynomial_in_evaluation_form,
        &blob_to_kzg_commitment,
        &compute_kzg_proof,
        &verify_kzg_proof,
    );
}

#[test]
pub fn test_eip4844_simple_interface() {
    assert!(init(CurveType::BLS12_381));

    eip4844_test(
        &load_trusted_setup,
        &blob_to_kzg_commitment,
        &compute_aggregate_kzg_proof,
        &verify_aggregate_kzg_proof,
    );
}

#[test]
pub fn test_blob_to_kzg_commitment() {
    assert!(init(CurveType::BLS12_381));

    blob_to_kzg_commitment_test(&load_trusted_setup, &blob_to_kzg_commitment, &bytes_from_g1)
}

#[test]
pub fn test_aggregate_proof_for_single_blob() {
    assert!(init(CurveType::BLS12_381));

    aggregate_proof_for_single_blob_test(&load_trusted_setup, &blob_to_kzg_commitment, &compute_aggregate_kzg_proof, &verify_aggregate_kzg_proof);
}

#[test]
pub fn compute_aggregate_kzg_proof_test_empty_() {
    assert!(init(CurveType::BLS12_381));

    compute_aggregate_kzg_proof_test_empty(
        &load_trusted_setup,
        &compute_aggregate_kzg_proof,
        &bytes_from_g1,
    )
}
