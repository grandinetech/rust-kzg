#![allow(unused)]

use crate::test_vectors::{
    blob_to_kzg_commitment, compute_blob_kzg_proof, compute_kzg_proof, verify_blob_kzg_proof,
    verify_blob_kzg_proof_batch, verify_kzg_proof,
};
use crate::tests::utils::{get_manifest_dir, get_trusted_setup_path};
use kzg::eip_4844::{
    BYTES_PER_BLOB, BYTES_PER_COMMITMENT, BYTES_PER_FIELD_ELEMENT, BYTES_PER_PROOF,
    FIELD_ELEMENTS_PER_BLOB, TRUSTED_SETUP_PATH,
};
use kzg::{FFTSettings, Fr, KZGSettings, Poly, G1, G2};
use pathdiff::diff_paths;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::env::current_dir;
use std::fs;
use std::path::PathBuf;

fn u64_to_bytes(x: u64) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[24..32].copy_from_slice(&x.to_be_bytes());
    bytes
}

pub fn generate_random_blob_bytes(rng: &mut ThreadRng) -> [u8; BYTES_PER_BLOB] {
    let mut arr = [0u8; BYTES_PER_BLOB];
    rng.fill(&mut arr[..]);
    // Ensure that the blob is canonical by ensuring that
    // each field element contained in the blob is < BLS_MODULUS
    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        arr[i * BYTES_PER_FIELD_ELEMENT] = 0;
    }
    arr
}

pub fn generate_random_field_element_bytes(rng: &mut ThreadRng) -> [u8; BYTES_PER_FIELD_ELEMENT] {
    let mut arr = [0u8; BYTES_PER_FIELD_ELEMENT];
    rng.fill(&mut arr[..]);
    // Ensure that the field element is canonical, i.e. < BLS_MODULUS
    arr[0] = 0;
    arr
}

#[rustfmt::skip]
const EXPECTED_POWERS: [[u64; 4usize]; 11] = [
    [1, 0, 0, 0],
    [32930439, 0, 0, 0],
    [1084413812732721, 0, 0, 0],
    [15773128324309817559, 1935, 0, 0],
    [17639716667354648417, 63748557064, 0, 0],
    [14688837229838358055, 2099267969765560859, 0, 0],
    [17806839894568993937, 15217493595388594120, 3747534, 0],
    [17407663719861420663, 10645919139951883969, 123407966953127, 0],
    [9882663619548185281, 9079722283539367550, 5594831647882181930, 220],
    [4160126872399834567, 5941227867469556516, 11658769961926678707, 7254684264],
    [4000187329613806065, 4317886535621327299, 17988956659770583631, 238899937640724696],
];

pub fn bytes_to_bls_field_test<TFr: Fr>() {
    let x: u64 = 329;
    let x_bytes = u64_to_bytes(x);
    let x_fr = TFr::from_bytes(&x_bytes).unwrap();

    assert_eq!(x_fr.to_bytes(), x_bytes);
    assert_eq!(x, x_fr.to_u64_arr()[0]);
}

pub fn compute_powers_test<TFr: Fr>(compute_powers: &dyn Fn(&TFr, usize) -> Vec<TFr>) {
    let x: u64 = 32930439;
    let n = 11;

    let x_bytes: [u8; 32] = u64_to_bytes(x);
    let x_fr = TFr::from_bytes(&x_bytes).unwrap();
    let powers = compute_powers(&x_fr, n);

    for (p, expected_p) in powers.iter().zip(EXPECTED_POWERS.iter()) {
        assert_eq!(expected_p, &p.to_u64_arr());
    }
}

#[allow(clippy::type_complexity)]
pub fn blob_to_kzg_commitment_test<
    TFr: Fr + Copy,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    load_trusted_setup: &dyn Fn(&str) -> Result<TKZGSettings, String>,
    blob_to_kzg_commitment: &dyn Fn(&[TFr], &TKZGSettings) -> Result<TG1, String>,
) {
    let ts = load_trusted_setup(get_trusted_setup_path().as_str()).unwrap();

    let field_element =
        TFr::from_hex("0x14629a3a39f7b854e6aa49aa2edb450267eac2c14bb2d4f97a0b81a3f57055ad")
            .unwrap();

    // Initialize the blob with a single field element
    let mut blob: [TFr; FIELD_ELEMENTS_PER_BLOB] = [TFr::zero(); FIELD_ELEMENTS_PER_BLOB];
    blob[0] = field_element;

    // Get a commitment to this particular blob
    let commitment = blob_to_kzg_commitment(&blob, &ts).unwrap();

    // We expect the commitment to match
    // If it doesn't match, something important has changed
    let expected_commitment = if cfg!(feature = "minimal-spec") {
        TG1::from_hex(
            "0x95d2d20379b60c353a9c2c75333a5d7d26d5ef5137c5200b\
            51bc9d0fd82d0270e98ac9d41a44c366684089e385e815e6",
        )
        .unwrap()
    } else {
        TG1::from_hex(
            "0x9815ded2101b6d233fdf31d826ba0557778506df8526f42a\
            87ccd82db36a238b50f8965c25d4484782097436d29e458e",
        )
        .unwrap()
    };

    assert!(commitment.equals(&expected_commitment));
}

#[allow(clippy::type_complexity)]
pub fn compute_kzg_proof_test<
    TFr: Fr + Copy,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    load_trusted_setup: &dyn Fn(&str) -> Result<TKZGSettings, String>,
    compute_kzg_proof: &dyn Fn(&[TFr], &TFr, &TKZGSettings) -> Result<(TG1, TFr), String>,
    blob_to_polynomial: &dyn Fn(&[TFr]) -> Result<TPoly, String>,
    evaluate_polynomial_in_evaluation_form: &dyn Fn(
        &TPoly,
        &TFr,
        &TKZGSettings,
    ) -> Result<TFr, String>,
) {
    let ts = load_trusted_setup(get_trusted_setup_path().as_str()).unwrap();

    let field_element =
        TFr::from_hex("0x69386e69dbae0357b399b8d645a57a3062dfbe00bd8e97170b9bdd6bc6168a13")
            .unwrap();
    let input_value =
        TFr::from_hex("0x03ea4fb841b4f9e01aa917c5e40dbd67efb4b8d4d9052069595f0647feba320d")
            .unwrap();

    // Initialize the blob with a single field element
    let mut blob: [TFr; FIELD_ELEMENTS_PER_BLOB] = [TFr::zero(); FIELD_ELEMENTS_PER_BLOB];
    blob[0] = field_element;

    // Compute the KZG proof for the given blob & z
    let (proof, output_value) = compute_kzg_proof(&blob, &input_value, &ts).unwrap();

    // Compare the computed proof to the expected proof
    let expected_proof = if cfg!(feature = "minimal-spec") {
        TG1::from_hex(
            "0xa846d83184f6d5b67bbbe905a875f6cfaf1c905e527ea49c\
            0616992fb8cce56d202c702b83d6fbe1fa75cacb050ffc27",
        )
        .unwrap()
    } else {
        TG1::from_hex(
            "0x899b7e1e7ff2e9b28c631d2f9d6b9ae828749c9dbf84f3f4\
            3b910bda9558f360f2fa0dac1143460b55908406038eb538",
        )
        .unwrap()
    };

    assert!(proof.equals(&expected_proof));

    // Get the expected y by evaluating the polynomial at input_value
    let poly = blob_to_polynomial(&blob).unwrap();
    let expected_output_value =
        evaluate_polynomial_in_evaluation_form(&poly, &input_value, &ts).unwrap();

    assert!(output_value.equals(&expected_output_value));
}

#[allow(clippy::type_complexity)]
pub fn compute_and_verify_kzg_proof_round_trip_test<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    load_trusted_setup: &dyn Fn(&str) -> Result<TKZGSettings, String>,
    blob_to_kzg_commitment: &dyn Fn(&[TFr], &TKZGSettings) -> Result<TG1, String>,
    bytes_to_blob: &dyn Fn(&[u8]) -> Result<Vec<TFr>, String>,
    compute_kzg_proof: &dyn Fn(&[TFr], &TFr, &TKZGSettings) -> Result<(TG1, TFr), String>,
    blob_to_polynomial: &dyn Fn(&[TFr]) -> Result<TPoly, String>,
    evaluate_polynomial_in_evaluation_form: &dyn Fn(
        &TPoly,
        &TFr,
        &TKZGSettings,
    ) -> Result<TFr, String>,
    verify_kzg_proof: &dyn Fn(&TG1, &TFr, &TFr, &TG1, &TKZGSettings) -> Result<bool, String>,
) {
    let ts = load_trusted_setup(get_trusted_setup_path().as_str()).unwrap();
    let mut rng = rand::thread_rng();

    let z_fr = {
        let z_bytes = generate_random_field_element_bytes(&mut rng);
        TFr::from_bytes(&z_bytes).unwrap()
    };

    let blob = {
        let blob_bytes = generate_random_blob_bytes(&mut rng);
        bytes_to_blob(&blob_bytes).unwrap()
    };

    // Get a commitment to that particular blob
    let commitment = blob_to_kzg_commitment(&blob, &ts).unwrap();

    // Compute the proof
    let (proof, computed_y) = compute_kzg_proof(&blob, &z_fr, &ts).unwrap();

    // Now let's attempt to verify the proof
    // First convert the blob to field elements
    let poly = blob_to_polynomial(&blob).unwrap();

    // Now evaluate the poly at `z` to learn `y`
    let y_fr = evaluate_polynomial_in_evaluation_form(&poly, &z_fr, &ts).unwrap();

    // Compare the recently evaluated y to the computed y
    assert!(y_fr.equals(&computed_y));

    // Finally verify the proof
    let result = verify_kzg_proof(&commitment, &z_fr, &y_fr, &proof, &ts).unwrap();
    assert!(result);
}

#[cfg(not(feature = "minimal-spec"))]
#[allow(clippy::type_complexity)]
pub fn compute_and_verify_kzg_proof_within_domain_test<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    load_trusted_setup: &dyn Fn(&str) -> Result<TKZGSettings, String>,
    blob_to_kzg_commitment: &dyn Fn(&[TFr], &TKZGSettings) -> Result<TG1, String>,
    bytes_to_blob: &dyn Fn(&[u8]) -> Result<Vec<TFr>, String>,
    compute_kzg_proof: &dyn Fn(&[TFr], &TFr, &TKZGSettings) -> Result<(TG1, TFr), String>,
    blob_to_polynomial: &dyn Fn(&[TFr]) -> Result<TPoly, String>,
    evaluate_polynomial_in_evaluation_form: &dyn Fn(
        &TPoly,
        &TFr,
        &TKZGSettings,
    ) -> Result<TFr, String>,
    verify_kzg_proof: &dyn Fn(&TG1, &TFr, &TFr, &TG1, &TKZGSettings) -> Result<bool, String>,
) {
    let ts = load_trusted_setup(get_trusted_setup_path().as_str()).unwrap();
    let mut rng = rand::thread_rng();

    for i in 0..25 {
        let blob = {
            let blob_bytes = generate_random_blob_bytes(&mut rng);
            bytes_to_blob(&blob_bytes).unwrap()
        };

        // Get a commitment to that particular blob
        let commitment = blob_to_kzg_commitment(&blob, &ts).unwrap();

        // Get the polynomial version of the blob
        let poly = blob_to_polynomial(&blob).unwrap();

        // Compute the proof
        let z_fr = ts.get_roots_of_unity_at(i);
        let (proof, computed_y) = compute_kzg_proof(&blob, &z_fr, &ts).unwrap();

        // Now evaluate the poly at `z` to learn `y`
        let y_fr = evaluate_polynomial_in_evaluation_form(&poly, &z_fr, &ts).unwrap();

        // Compare the recently evaluated y to the computed y
        assert!(y_fr.equals(&computed_y));

        // Finally verify the proof
        let result = verify_kzg_proof(&commitment, &z_fr, &y_fr, &proof, &ts).unwrap();
        assert!(result);
    }
}

#[allow(clippy::type_complexity)]
pub fn compute_and_verify_kzg_proof_fails_with_incorrect_proof_test<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    load_trusted_setup: &dyn Fn(&str) -> Result<TKZGSettings, String>,
    blob_to_kzg_commitment: &dyn Fn(&[TFr], &TKZGSettings) -> Result<TG1, String>,
    bytes_to_blob: &dyn Fn(&[u8]) -> Result<Vec<TFr>, String>,
    compute_kzg_proof: &dyn Fn(&[TFr], &TFr, &TKZGSettings) -> Result<(TG1, TFr), String>,
    blob_to_polynomial: &dyn Fn(&[TFr]) -> Result<TPoly, String>,
    evaluate_polynomial_in_evaluation_form: &dyn Fn(
        &TPoly,
        &TFr,
        &TKZGSettings,
    ) -> Result<TFr, String>,
    verify_kzg_proof: &dyn Fn(&TG1, &TFr, &TFr, &TG1, &TKZGSettings) -> Result<bool, String>,
) {
    let ts = load_trusted_setup(get_trusted_setup_path().as_str()).unwrap();
    let mut rng = rand::thread_rng();

    let z_fr = {
        let z_bytes = generate_random_field_element_bytes(&mut rng);
        TFr::from_bytes(&z_bytes).unwrap()
    };

    let blob = {
        let blob_bytes = generate_random_blob_bytes(&mut rng);
        bytes_to_blob(&blob_bytes).unwrap()
    };

    // Get a commitment to that particular blob
    let commitment = blob_to_kzg_commitment(&blob, &ts).unwrap();

    // Compute the proof
    let (mut proof, _) = compute_kzg_proof(&blob, &z_fr, &ts).unwrap();

    // Now let's attempt to verify the proof
    // First convert the blob to field elements
    let poly = blob_to_polynomial(&blob).unwrap();

    // Now evaluate the poly at `z` to learn `y`
    let y_fr = evaluate_polynomial_in_evaluation_form(&poly, &z_fr, &ts).unwrap();

    // Change the proof so it should not verify
    proof = proof.add(&TG1::generator());

    // Finally verify the proof
    let result = verify_kzg_proof(&commitment, &z_fr, &y_fr, &proof, &ts).unwrap();
    assert!(!result);
}

#[allow(clippy::type_complexity)]
pub fn compute_and_verify_blob_kzg_proof_test<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    load_trusted_setup: &dyn Fn(&str) -> Result<TKZGSettings, String>,
    blob_to_kzg_commitment: &dyn Fn(&[TFr], &TKZGSettings) -> Result<TG1, String>,
    bytes_to_blob: &dyn Fn(&[u8]) -> Result<Vec<TFr>, String>,
    compute_blob_kzg_proof: &dyn Fn(&[TFr], &TG1, &TKZGSettings) -> Result<TG1, String>,
    verify_blob_kzg_proof: &dyn Fn(&[TFr], &TG1, &TG1, &TKZGSettings) -> Result<bool, String>,
) {
    let ts = load_trusted_setup(get_trusted_setup_path().as_str()).unwrap();
    let mut rng = rand::thread_rng();

    // Some preparation
    let blob = {
        let blob_bytes = generate_random_blob_bytes(&mut rng);
        bytes_to_blob(&blob_bytes).unwrap()
    };

    // Compute the proof
    let commitment = blob_to_kzg_commitment(&blob, &ts).unwrap();
    let proof = compute_blob_kzg_proof(&blob, &commitment, &ts).unwrap();

    // Finally verify the proof
    let result = verify_blob_kzg_proof(&blob, &commitment, &proof, &ts).unwrap();
    assert!(result);
}

#[allow(clippy::type_complexity)]
pub fn compute_and_verify_blob_kzg_proof_fails_with_incorrect_proof_test<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    load_trusted_setup: &dyn Fn(&str) -> Result<TKZGSettings, String>,
    blob_to_kzg_commitment: &dyn Fn(&[TFr], &TKZGSettings) -> Result<TG1, String>,
    bytes_to_blob: &dyn Fn(&[u8]) -> Result<Vec<TFr>, String>,
    compute_blob_kzg_proof: &dyn Fn(&[TFr], &TG1, &TKZGSettings) -> Result<TG1, String>,
    verify_blob_kzg_proof: &dyn Fn(&[TFr], &TG1, &TG1, &TKZGSettings) -> Result<bool, String>,
) {
    let ts = load_trusted_setup(get_trusted_setup_path().as_str()).unwrap();
    let mut rng = rand::thread_rng();

    // Some preparation
    let blob = {
        let blob_bytes = generate_random_blob_bytes(&mut rng);
        bytes_to_blob(&blob_bytes).unwrap()
    };

    // Compute the proof
    let commitment = blob_to_kzg_commitment(&blob, &ts).unwrap();
    let mut proof = compute_blob_kzg_proof(&blob, &commitment, &ts).unwrap();

    // Change the proof so it should not verify
    proof = proof.add(&TG1::generator());

    // Finally verify the proof
    let result = verify_blob_kzg_proof(&blob, &commitment, &proof, &ts).unwrap();
    assert!(!result);
}

#[allow(clippy::type_complexity)]
pub fn verify_kzg_proof_batch_test<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    load_trusted_setup: &dyn Fn(&str) -> Result<TKZGSettings, String>,
    blob_to_kzg_commitment: &dyn Fn(&[TFr], &TKZGSettings) -> Result<TG1, String>,
    bytes_to_blob: &dyn Fn(&[u8]) -> Result<Vec<TFr>, String>,
    compute_blob_kzg_proof: &dyn Fn(&[TFr], &TG1, &TKZGSettings) -> Result<TG1, String>,
    verify_blob_kzg_proof_batch: &dyn Fn(
        &[Vec<TFr>],
        &[TG1],
        &[TG1],
        &TKZGSettings,
    ) -> Result<bool, String>,
) {
    let ts = load_trusted_setup(get_trusted_setup_path().as_str()).unwrap();
    let mut rng = rand::thread_rng();

    const N_SAMPLES: usize = 16;

    let mut blobs: Vec<Vec<TFr>> = Vec::new();
    let mut commitments: Vec<TG1> = Vec::new();
    let mut proofs: Vec<TG1> = Vec::new();

    // Some preparation
    for _ in 0..N_SAMPLES {
        let blob = {
            let blob_bytes = generate_random_blob_bytes(&mut rng);
            bytes_to_blob(&blob_bytes).unwrap()
        };

        let commitment = blob_to_kzg_commitment(&blob, &ts).unwrap();
        let proof = compute_blob_kzg_proof(&blob, &commitment, &ts).unwrap();

        blobs.push(blob);
        commitments.push(commitment);
        proofs.push(proof);
    }

    // Verify batched proofs for 0,1,2..16 blobs
    // This should still work with zero blobs
    for count in 0..(N_SAMPLES + 1) {
        let result = verify_blob_kzg_proof_batch(
            &blobs[0..count],
            &commitments[0..count],
            &proofs[0..count],
            &ts,
        )
        .unwrap();
        assert!(result);
    }
}

#[allow(clippy::type_complexity)]
pub fn verify_kzg_proof_batch_fails_with_incorrect_proof_test<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    load_trusted_setup: &dyn Fn(&str) -> Result<TKZGSettings, String>,
    blob_to_kzg_commitment: &dyn Fn(&[TFr], &TKZGSettings) -> Result<TG1, String>,
    bytes_to_blob: &dyn Fn(&[u8]) -> Result<Vec<TFr>, String>,
    compute_blob_kzg_proof: &dyn Fn(&[TFr], &TG1, &TKZGSettings) -> Result<TG1, String>,
    verify_blob_kzg_proof_batch: &dyn Fn(
        &[Vec<TFr>],
        &[TG1],
        &[TG1],
        &TKZGSettings,
    ) -> Result<bool, String>,
) {
    let ts = load_trusted_setup(get_trusted_setup_path().as_str()).unwrap();
    let mut rng = rand::thread_rng();

    const N_SAMPLES: usize = 2;

    let mut blobs: Vec<Vec<TFr>> = Vec::new();
    let mut commitments: Vec<TG1> = Vec::new();
    let mut proofs: Vec<TG1> = Vec::new();

    // Some preparation
    for _ in 0..N_SAMPLES {
        let blob = {
            let blob_bytes = generate_random_blob_bytes(&mut rng);
            bytes_to_blob(&blob_bytes).unwrap()
        };

        let commitment = blob_to_kzg_commitment(&blob, &ts).unwrap();
        let proof = compute_blob_kzg_proof(&blob, &commitment, &ts).unwrap();

        blobs.push(blob);
        commitments.push(commitment);
        proofs.push(proof);
    }

    // Overwrite second proof with an incorrect one
    proofs[1] = proofs[0].clone();

    let result = verify_blob_kzg_proof_batch(&blobs, &commitments, &proofs, &ts).unwrap();
    assert!(!result);
}

const BLOB_TO_KZG_COMMITMENT_TESTS: &str = "src/test_vectors/blob_to_kzg_commitment/*/*/*";
const COMPUTE_KZG_PROOF_TESTS: &str = "src/test_vectors/compute_kzg_proof/*/*/*";
const COMPUTE_BLOB_KZG_PROOF_TESTS: &str = "src/test_vectors/compute_blob_kzg_proof/*/*/*";
const VERIFY_KZG_PROOF_TESTS: &str = "src/test_vectors/verify_kzg_proof/*/*/*";
const VERIFY_BLOB_KZG_PROOF_TESTS: &str = "src/test_vectors/verify_blob_kzg_proof/*/*/*";
const VERIFY_BLOB_KZG_PROOF_BATCH_TESTS: &str =
    "src/test_vectors/verify_blob_kzg_proof_batch/*/*/*";

#[cfg(not(feature = "minimal-spec"))]
#[allow(clippy::type_complexity)]
pub fn test_vectors_blob_to_kzg_commitment<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    load_trusted_setup: &dyn Fn(&str) -> Result<TKZGSettings, String>,
    blob_to_kzg_commitment: &dyn Fn(&[TFr], &TKZGSettings) -> Result<TG1, String>,
    bytes_to_blob: &dyn Fn(&[u8]) -> Result<Vec<TFr>, String>,
) {
    let ts = load_trusted_setup(get_trusted_setup_path().as_str()).unwrap();
    let test_files: Vec<PathBuf> = glob::glob(&format!(
        "{}/{}",
        get_manifest_dir(),
        BLOB_TO_KZG_COMMITMENT_TESTS
    ))
    .unwrap()
    .map(Result::unwrap)
    .collect();
    assert!(!test_files.is_empty());

    for test_file in test_files {
        let yaml_data = fs::read_to_string(test_file).unwrap();
        let test: blob_to_kzg_commitment::Test = serde_yaml::from_str(&yaml_data).unwrap();

        let blob = match bytes_to_blob(&test.input.get_blob_bytes()) {
            Ok(blob) => blob,
            Err(_) => {
                assert!(test.get_output_bytes().is_none());
                continue;
            }
        };

        let expected_commitment = {
            let commitment_bytes = test.get_output_bytes().unwrap();
            TG1::from_bytes(&commitment_bytes).unwrap()
        };

        let commitment = blob_to_kzg_commitment(&blob, &ts).unwrap();
        assert!(commitment.equals(&expected_commitment));
    }
}

#[cfg(not(feature = "minimal-spec"))]
#[allow(clippy::type_complexity)]
pub fn test_vectors_compute_kzg_proof<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    load_trusted_setup: &dyn Fn(&str) -> Result<TKZGSettings, String>,
    compute_kzg_proof: &dyn Fn(&[TFr], &TFr, &TKZGSettings) -> Result<(TG1, TFr), String>,
    bytes_to_blob: &dyn Fn(&[u8]) -> Result<Vec<TFr>, String>,
) {
    let ts = load_trusted_setup(get_trusted_setup_path().as_str()).unwrap();
    let test_files: Vec<PathBuf> = glob::glob(&format!(
        "{}/{}",
        get_manifest_dir(),
        COMPUTE_KZG_PROOF_TESTS
    ))
    .unwrap()
    .map(Result::unwrap)
    .collect();
    assert!(!test_files.is_empty());

    for test_file in test_files {
        let yaml_data = fs::read_to_string(test_file).unwrap();
        let test: compute_kzg_proof::Test = serde_yaml::from_str(&yaml_data).unwrap();

        let input = (
            match bytes_to_blob(&test.input.get_blob_bytes()) {
                Ok(blob) => blob,
                Err(_) => {
                    assert!(test.get_output_bytes().is_none());
                    continue;
                }
            },
            match TFr::from_bytes(&test.input.get_z_bytes()) {
                Ok(z) => z,
                Err(_) => {
                    assert!(test.get_output_bytes().is_none());
                    continue;
                }
            },
        );

        let output = (
            test.get_output_bytes()
                .and_then(|bytes| TG1::from_bytes(&bytes.0).ok()), // proof
            test.get_output_bytes()
                .and_then(|bytes| TFr::from_bytes(&bytes.1).ok()), // y
        );

        // Compute the proof
        let (proof, y) = compute_kzg_proof(&input.0, &input.1, &ts).unwrap();

        // Compare the computed and expected proofs
        assert!(proof.equals(&output.0.unwrap()));

        // Compare the computed and expected ys
        assert!(y.equals(&output.1.unwrap()));
    }
}

#[cfg(not(feature = "minimal-spec"))]
#[allow(clippy::type_complexity)]
pub fn test_vectors_compute_blob_kzg_proof<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    load_trusted_setup: &dyn Fn(&str) -> Result<TKZGSettings, String>,
    bytes_to_blob: &dyn Fn(&[u8]) -> Result<Vec<TFr>, String>,
    compute_blob_kzg_proof: &dyn Fn(&[TFr], &TG1, &TKZGSettings) -> Result<TG1, String>,
) {
    let ts = load_trusted_setup(get_trusted_setup_path().as_str()).unwrap();
    let test_files: Vec<PathBuf> = glob::glob(&format!(
        "{}/{}",
        get_manifest_dir(),
        COMPUTE_BLOB_KZG_PROOF_TESTS
    ))
    .unwrap()
    .map(Result::unwrap)
    .collect();
    assert!(!test_files.is_empty());

    for test_file in test_files {
        let yaml_data = fs::read_to_string(test_file).unwrap();
        let test: compute_blob_kzg_proof::Test = serde_yaml::from_str(&yaml_data).unwrap();

        let input = (
            match bytes_to_blob(&test.input.get_blob_bytes()) {
                Ok(blob) => blob,
                Err(_) => {
                    assert!(test.get_output_bytes().is_none());
                    continue;
                }
            },
            match TG1::from_bytes(&test.input.get_commitment_bytes()) {
                Ok(commitment) => commitment,
                Err(_) => {
                    assert!(test.get_output_bytes().is_none());
                    continue;
                }
            },
        );

        match compute_blob_kzg_proof(&input.0, &input.1, &ts) {
            Ok(proof) => {
                let expected_commitment = test
                    .get_output_bytes()
                    .and_then(|commitment_bytes| TG1::from_bytes(&commitment_bytes).ok());

                assert!(proof.equals(&expected_commitment.unwrap_or_default()));
            }
            Err(_) => {
                assert!(test.get_output_bytes().is_none());
                continue;
            }
        };
    }
}

#[cfg(not(feature = "minimal-spec"))]
#[allow(clippy::type_complexity)]
pub fn test_vectors_verify_kzg_proof<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    load_trusted_setup: &dyn Fn(&str) -> Result<TKZGSettings, String>,
    verify_kzg_proof: &dyn Fn(&TG1, &TFr, &TFr, &TG1, &TKZGSettings) -> Result<bool, String>,
) {
    let ts = load_trusted_setup(get_trusted_setup_path().as_str()).unwrap();
    let test_files: Vec<PathBuf> = glob::glob(&format!(
        "{}/{}",
        get_manifest_dir(),
        VERIFY_KZG_PROOF_TESTS
    ))
    .unwrap()
    .map(Result::unwrap)
    .collect();
    assert!(!test_files.is_empty());

    for test_file in test_files {
        let yaml_data = fs::read_to_string(test_file).unwrap();
        let test: verify_kzg_proof::Test = serde_yaml::from_str(&yaml_data).unwrap();

        let input = (
            match TG1::from_bytes(&test.input.get_commitment_bytes()) {
                Ok(commitment) => commitment,
                Err(_) => {
                    assert!(test.get_output().is_none());
                    continue;
                }
            },
            match TFr::from_bytes(&test.input.get_z_bytes()) {
                Ok(z) => z,
                Err(_) => {
                    assert!(test.get_output().is_none());
                    continue;
                }
            },
            match TFr::from_bytes(&test.input.get_y_bytes()) {
                Ok(y) => y,
                Err(_) => {
                    assert!(test.get_output().is_none());
                    continue;
                }
            },
            match TG1::from_bytes(&test.input.get_proof_bytes()) {
                Ok(proof) => proof,
                Err(_) => {
                    assert!(test.get_output().is_none());
                    continue;
                }
            },
        );

        match verify_kzg_proof(&input.0, &input.1, &input.2, &input.3, &ts) {
            Ok(result) => assert_eq!(result, test.get_output().unwrap()),
            Err(_) => {
                assert!(test.get_output().is_none());
                continue;
            }
        };
    }
}

#[cfg(not(feature = "minimal-spec"))]
#[allow(clippy::type_complexity)]
pub fn test_vectors_verify_blob_kzg_proof<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    load_trusted_setup: &dyn Fn(&str) -> Result<TKZGSettings, String>,
    bytes_to_blob: &dyn Fn(&[u8]) -> Result<Vec<TFr>, String>,
    verify_blob_kzg_proof: &dyn Fn(&[TFr], &TG1, &TG1, &TKZGSettings) -> Result<bool, String>,
) {
    let ts = load_trusted_setup(get_trusted_setup_path().as_str()).unwrap();
    let test_files: Vec<PathBuf> = glob::glob(&format!(
        "{}/{}",
        get_manifest_dir(),
        VERIFY_BLOB_KZG_PROOF_TESTS
    ))
    .unwrap()
    .map(Result::unwrap)
    .collect();
    assert!(!test_files.is_empty());

    for test_file in test_files {
        let yaml_data = fs::read_to_string(test_file).unwrap();
        let test: verify_blob_kzg_proof::Test = serde_yaml::from_str(&yaml_data).unwrap();

        let input = (
            match bytes_to_blob(&test.input.get_blob_bytes()) {
                Ok(blob) => blob,
                Err(_) => {
                    assert!(test.get_output().is_none());
                    continue;
                }
            },
            match TG1::from_bytes(&test.input.get_commitment_bytes()) {
                Ok(commitment) => commitment,
                Err(_) => {
                    assert!(test.get_output().is_none());
                    continue;
                }
            },
            match TG1::from_bytes(&test.input.get_proof_bytes()) {
                Ok(proof) => proof,
                Err(_) => {
                    assert!(test.get_output().is_none());
                    continue;
                }
            },
        );

        match verify_blob_kzg_proof(&input.0, &input.1, &input.2, &ts) {
            Ok(result) => assert_eq!(result, test.get_output().unwrap()),
            Err(_) => {
                assert!(test.get_output().is_none());
                continue;
            }
        };
    }
}

#[cfg(not(feature = "minimal-spec"))]
#[allow(clippy::type_complexity)]
pub fn test_vectors_verify_blob_kzg_proof_batch<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    load_trusted_setup: &dyn Fn(&str) -> Result<TKZGSettings, String>,
    bytes_to_blob: &dyn Fn(&[u8]) -> Result<Vec<TFr>, String>,
    verify_blob_kzg_proof_batch: &dyn Fn(
        &[Vec<TFr>],
        &[TG1],
        &[TG1],
        &TKZGSettings,
    ) -> Result<bool, String>,
) {
    let ts = load_trusted_setup(get_trusted_setup_path().as_str()).unwrap();
    let test_files: Vec<PathBuf> = glob::glob(&format!(
        "{}/{}",
        get_manifest_dir(),
        VERIFY_BLOB_KZG_PROOF_BATCH_TESTS
    ))
    .unwrap()
    .map(Result::unwrap)
    .collect();
    assert!(!test_files.is_empty());

    for test_file in test_files {
        let yaml_data = fs::read_to_string(test_file).unwrap();
        let test: verify_blob_kzg_proof_batch::Test = serde_yaml::from_str(&yaml_data).unwrap();

        let mut skip_test = false;

        let mut blobs = Vec::new();
        let mut commitments = Vec::new();
        let mut proofs = Vec::new();

        for blob_bytes in test.input.get_blobs_bytes() {
            if let Ok(blob) = bytes_to_blob(blob_bytes.as_slice()) {
                blobs.push(blob);
            } else {
                assert!(test.get_output().is_none());
                skip_test = true;
                continue;
            }
        }

        for commitment_bytes in test.input.get_commitments_bytes() {
            if let Ok(commitment) = TG1::from_bytes(commitment_bytes.as_slice()) {
                commitments.push(commitment);
            } else {
                assert!(test.get_output().is_none());
                skip_test = true;
                continue;
            }
        }

        for proof_bytes in test.input.get_proofs_bytes() {
            if let Ok(proof) = TG1::from_bytes(proof_bytes.as_slice()) {
                proofs.push(proof);
            } else {
                assert!(test.get_output().is_none());
                skip_test = true;
                continue;
            }
        }

        if skip_test {
            assert!(test.get_output().is_none());
            continue;
        }

        match verify_blob_kzg_proof_batch(&blobs, &commitments, &proofs, &ts) {
            Ok(result) => assert_eq!(result, test.get_output().unwrap()),
            Err(_) => {
                assert!(test.get_output().is_none());
                continue;
            }
        };
    }
}
