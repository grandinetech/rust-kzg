use ark_poly::DenseUVPolynomial;
use kzg::{FFTG1, G1Mul};
use kzg::common_utils::{reverse_bit_order, compute_challenge, hash_to_bls_field, compute_powers, validate_batched_input, compute_r_powers, evaluate_polynomial_in_evaluation_form, blob_to_polynomial, load_trusted_setup_rust, fr_batch_inv};
use crate::fft_g1::g1_linear_combination;
use crate::kzg_proofs::{pairings_verify, FFTSettings, KZGSettings};
use crate::kzg_types::{ArkG1, ArkG2, ArkFr};
use crate::utils::{PolyData};
use ark_bls12_381::{Bls12_381, g1};
use ark_std::test_rng;
use kzg::eip_4844::{
    bytes_of_uint64, hash, load_trusted_setup_string, BYTES_PER_BLOB, BYTES_PER_COMMITMENT,
    BYTES_PER_FIELD_ELEMENT, BYTES_PER_G1, BYTES_PER_G2, BYTES_PER_PROOF, CHALLENGE_INPUT_SIZE,
    FIAT_SHAMIR_PROTOCOL_DOMAIN, FIELD_ELEMENTS_PER_BLOB, RANDOM_CHALLENGE_KZG_BATCH_DOMAIN,
    TRUSTED_SETUP_NUM_G2_POINTS,
};
use kzg::{cfg_into_iter, FFTSettings as FFTSettingsT, Fr, KZGSettings as LKZGSettings, G2};
use kzg::{Poly, G1};
use std::fs::File;
use std::io::Read;
use ark_ec::CurveGroup;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

pub fn load_trusted_setup(filepath: &str) -> Result<KZGSettings, String> {
    let mut file = File::open(filepath).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    let (g1_bytes, g2_bytes) = load_trusted_setup_string(&contents)?;
    load_trusted_setup_rust(g1_bytes.as_slice(), g2_bytes.as_slice())
}
