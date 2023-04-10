use kzg::eip_4844::{
    BYTES_PER_BLOB, BYTES_PER_FIELD_ELEMENT, FIELD_ELEMENTS_PER_BLOB, TRUSTED_SETUP_PATH,
};
use kzg::{FFTSettings, Fr, KZGSettings, Poly, G1, G2};
use rand::rngs::ThreadRng;
use rand::Rng;
use std::env::set_current_dir;

fn u64_to_bytes(x: u64) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[0..8].copy_from_slice(&x.to_le_bytes());
    bytes
}

pub fn generate_random_blob_bytes(rng: &mut ThreadRng) -> [u8; BYTES_PER_BLOB] {
    let mut arr = [0u8; BYTES_PER_BLOB];
    rng.fill(&mut arr[..]);
    // Ensure that the blob is canonical by ensuring that
    // each field element contained in the blob is < BLS_MODULUS
    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        arr[i * BYTES_PER_FIELD_ELEMENT + BYTES_PER_FIELD_ELEMENT - 1] = 0;
    }
    arr
}

pub fn generate_random_field_element_bytes(rng: &mut ThreadRng) -> [u8; BYTES_PER_FIELD_ELEMENT] {
    let mut arr = [0u8; BYTES_PER_FIELD_ELEMENT];
    rng.fill(&mut arr[..]);
    // Ensure that the field element is canonical, i.e. < BLS_MODULUS
    arr[BYTES_PER_FIELD_ELEMENT - 1] = 0;
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

pub fn bytes_to_bls_field_test<TFr: Fr>(
    bytes_to_bls_field: &dyn Fn(&[u8; 32usize]) -> TFr,
    bytes_from_bls_field: &dyn Fn(&TFr) -> [u8; 32usize],
) {
    let x: u64 = 329;
    let x_bytes = u64_to_bytes(x);
    let x_fr = bytes_to_bls_field(&x_bytes);

    assert_eq!(bytes_from_bls_field(&x_fr), x_bytes);
    assert_eq!(x, x_fr.to_u64_arr()[0]);
}

pub fn compute_powers_test<TFr: Fr>(
    bytes_to_bls_field: &dyn Fn(&[u8; 32usize]) -> TFr,
    compute_powers: &dyn Fn(&TFr, usize) -> Vec<TFr>,
) {
    let x: u64 = 32930439;
    let n = 11;

    let x_bytes: [u8; 32] = u64_to_bytes(x);
    let x_fr = bytes_to_bls_field(&x_bytes);
    let powers = compute_powers(&x_fr, n);

    for (p, expected_p) in powers.iter().zip(EXPECTED_POWERS.iter()) {
        assert_eq!(expected_p, &p.to_u64_arr());
    }
}

pub fn blob_to_kzg_commitment_test<
    TFr: Fr + Copy,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    load_trusted_setup: &dyn Fn(&str) -> TKZGSettings,
    blob_to_kzg_commitment: &dyn Fn(&[TFr], &TKZGSettings) -> TG1,
    hex_to_bls_field: &dyn Fn(&str) -> TFr,
    hex_to_g1: &dyn Fn(&str) -> TG1,
) {
    set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
    let ts = load_trusted_setup(TRUSTED_SETUP_PATH);

    let field_element =
        hex_to_bls_field("ad5570f5a3810b7af9d4b24bc1c2ea670245db2eaa49aae654b8f7393a9a6214");

    // Initialize the blob with a single field element
    let mut blob: [TFr; FIELD_ELEMENTS_PER_BLOB] = [TFr::zero(); FIELD_ELEMENTS_PER_BLOB];
    blob[0] = field_element;

    // Get a commitment to this particular blob
    let commitment = blob_to_kzg_commitment(&blob, &ts);

    // We expect the commitment to match
    // If it doesn't match, something important has changed
    let expected_commitment = if cfg!(feature = "minimal-spec") {
        hex_to_g1(
            "95d2d20379b60c353a9c2c75333a5d7d26d5ef5137c5200b\
            51bc9d0fd82d0270e98ac9d41a44c366684089e385e815e6",
        )
    } else {
        hex_to_g1(
            "9815ded2101b6d233fdf31d826ba0557778506df8526f42a\
            87ccd82db36a238b50f8965c25d4484782097436d29e458e",
        )
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
    load_trusted_setup: &dyn Fn(&str) -> TKZGSettings,
    hex_to_bls_field: &dyn Fn(&str) -> TFr,
    hex_to_g1: &dyn Fn(&str) -> TG1,
    compute_kzg_proof: &dyn Fn(&[TFr], &TFr, &TKZGSettings) -> (TG1, TFr),
    blob_to_polynomial: &dyn Fn(&[TFr]) -> TPoly,
    evaluate_polynomial_in_evaluation_form: &dyn Fn(&TPoly, &TFr, &TKZGSettings) -> TFr,
) {
    set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
    let ts = load_trusted_setup(TRUSTED_SETUP_PATH);

    let field_element =
        hex_to_bls_field("138a16c66bdd9b0b17978ebd00bedf62307aa545d6b899b35703aedb696e3869");
    let input_value =
        hex_to_bls_field("0d32bafe47065f59692005d9d4b8b4ef67bd0de4c517a91ae0f9b441b84fea03");

    // Initialize the blob with a single field element
    let mut blob: [TFr; FIELD_ELEMENTS_PER_BLOB] = [TFr::zero(); FIELD_ELEMENTS_PER_BLOB];
    blob[0] = field_element;

    // Compute the KZG proof for the given blob & z
    let (proof, output_value) = compute_kzg_proof(&blob, &input_value, &ts);

    // Compare the computed proof to the expected proof
    let expected_proof = if cfg!(feature = "minimal-spec") {
        hex_to_g1(
            "a846d83184f6d5b67bbbe905a875f6cfaf1c905e527ea49c\
            0616992fb8cce56d202c702b83d6fbe1fa75cacb050ffc27",
        )
    } else {
        hex_to_g1(
            "899b7e1e7ff2e9b28c631d2f9d6b9ae828749c9dbf84f3f4\
            3b910bda9558f360f2fa0dac1143460b55908406038eb538",
        )
    };

    assert!(proof.equals(&expected_proof));

    // Get the expected y by evaluating the polynomial at input_value
    let poly = blob_to_polynomial(&blob);
    let expected_output_value = evaluate_polynomial_in_evaluation_form(&poly, &input_value, &ts);

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
    load_trusted_setup: &dyn Fn(&str) -> TKZGSettings,
    blob_to_kzg_commitment: &dyn Fn(&[TFr], &TKZGSettings) -> TG1,
    bytes_to_bls_field: &dyn Fn(&[u8; 32usize]) -> Result<TFr, u8>,
    compute_kzg_proof: &dyn Fn(&[TFr], &TFr, &TKZGSettings) -> (TG1, TFr),
    blob_to_polynomial: &dyn Fn(&[TFr]) -> TPoly,
    evaluate_polynomial_in_evaluation_form: &dyn Fn(&TPoly, &TFr, &TKZGSettings) -> TFr,
    verify_kzg_proof: &dyn Fn(&TG1, &TFr, &TFr, &TG1, &TKZGSettings) -> bool,
) {
    set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
    let ts = load_trusted_setup(TRUSTED_SETUP_PATH);
    let mut rng = rand::thread_rng();

    let z_fr = {
        let z_bytes = generate_random_field_element_bytes(&mut rng);
        bytes_to_bls_field(&z_bytes).unwrap()
    };

    let blob: Vec<TFr> = generate_random_blob_bytes(&mut rng)
        .chunks(BYTES_PER_FIELD_ELEMENT)
        .map(|chunk| {
            bytes_to_bls_field(
                chunk
                    .try_into()
                    .expect("Chunked into incorrect number of bytes"),
            )
            .unwrap()
        })
        .collect();

    // Get a commitment to that particular blob
    let commitment = blob_to_kzg_commitment(&blob, &ts);

    // Compute the proof
    let (proof, computed_y) = compute_kzg_proof(&blob, &z_fr, &ts);

    // Now let's attempt to verify the proof
    // First convert the blob to field elements
    let poly = blob_to_polynomial(&blob);

    // Now evaluate the poly at `z` to learn `y`
    let y_fr = evaluate_polynomial_in_evaluation_form(&poly, &z_fr, &ts);

    // Compare the recently evaluated y to the computed y
    assert!(y_fr.equals(&computed_y));

    // Finally verify the proof
    let result = verify_kzg_proof(&commitment, &z_fr, &y_fr, &proof, &ts);
    assert!(result);
}

#[allow(clippy::type_complexity)]
pub fn compute_and_verify_kzg_proof_within_domain_test<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    load_trusted_setup: &dyn Fn(&str) -> TKZGSettings,
    blob_to_kzg_commitment: &dyn Fn(&[TFr], &TKZGSettings) -> TG1,
    bytes_to_bls_field: &dyn Fn(&[u8; 32usize]) -> Result<TFr, u8>,
    compute_kzg_proof: &dyn Fn(&[TFr], &TFr, &TKZGSettings) -> (TG1, TFr),
    blob_to_polynomial: &dyn Fn(&[TFr]) -> TPoly,
    evaluate_polynomial_in_evaluation_form: &dyn Fn(&TPoly, &TFr, &TKZGSettings) -> TFr,
    verify_kzg_proof: &dyn Fn(&TG1, &TFr, &TFr, &TG1, &TKZGSettings) -> bool,
) {
    set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
    let ts = load_trusted_setup(TRUSTED_SETUP_PATH);
    let mut rng = rand::thread_rng();

    for i in 0..25 {
        let blob: Vec<TFr> = generate_random_blob_bytes(&mut rng)
            .chunks(BYTES_PER_FIELD_ELEMENT)
            .map(|chunk| {
                bytes_to_bls_field(
                    chunk
                        .try_into()
                        .expect("Chunked into incorrect number of bytes"),
                )
                .unwrap()
            })
            .collect();

        // Get a commitment to that particular blob
        let commitment = blob_to_kzg_commitment(&blob, &ts);

        // Get the polynomial version of the blob
        let poly = blob_to_polynomial(&blob);

        // Compute the proof
        let z_fr = ts.get_expanded_roots_of_unity_at(i);
        let (proof, computed_y) = compute_kzg_proof(&blob, &z_fr, &ts);

        // Now evaluate the poly at `z` to learn `y`
        let y_fr = evaluate_polynomial_in_evaluation_form(&poly, &z_fr, &ts);

        // Compare the recently evaluated y to the computed y
        assert!(y_fr.equals(&computed_y));

        // Finally verify the proof
        let result = verify_kzg_proof(&commitment, &z_fr, &y_fr, &proof, &ts);
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
    load_trusted_setup: &dyn Fn(&str) -> TKZGSettings,
    blob_to_kzg_commitment: &dyn Fn(&[TFr], &TKZGSettings) -> TG1,
    bytes_to_bls_field: &dyn Fn(&[u8; 32usize]) -> Result<TFr, u8>,
    compute_kzg_proof: &dyn Fn(&[TFr], &TFr, &TKZGSettings) -> (TG1, TFr),
    blob_to_polynomial: &dyn Fn(&[TFr]) -> TPoly,
    evaluate_polynomial_in_evaluation_form: &dyn Fn(&TPoly, &TFr, &TKZGSettings) -> TFr,
    verify_kzg_proof: &dyn Fn(&TG1, &TFr, &TFr, &TG1, &TKZGSettings) -> bool,
) {
    set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
    let ts = load_trusted_setup(TRUSTED_SETUP_PATH);
    let mut rng = rand::thread_rng();

    let z_fr = {
        let z_bytes = generate_random_field_element_bytes(&mut rng);
        bytes_to_bls_field(&z_bytes).unwrap()
    };

    let blob: Vec<TFr> = generate_random_blob_bytes(&mut rng)
        .chunks(BYTES_PER_FIELD_ELEMENT)
        .map(|chunk| {
            bytes_to_bls_field(
                chunk
                    .try_into()
                    .expect("Chunked into incorrect number of bytes"),
            )
            .unwrap()
        })
        .collect();

    // Get a commitment to that particular blob
    let commitment = blob_to_kzg_commitment(&blob, &ts);

    // Compute the proof
    let (mut proof, _) = compute_kzg_proof(&blob, &z_fr, &ts);

    // Now let's attempt to verify the proof
    // First convert the blob to field elements
    let poly = blob_to_polynomial(&blob);

    // Now evaluate the poly at `z` to learn `y`
    let y_fr = evaluate_polynomial_in_evaluation_form(&poly, &z_fr, &ts);

    // Change the proof so it should not verify
    proof = proof.add(&TG1::generator());

    // Finally verify the proof
    let result = verify_kzg_proof(&commitment, &z_fr, &y_fr, &proof, &ts);
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
    load_trusted_setup: &dyn Fn(&str) -> TKZGSettings,
    blob_to_kzg_commitment: &dyn Fn(&[TFr], &TKZGSettings) -> TG1,
    bytes_to_bls_field: &dyn Fn(&[u8; 32usize]) -> Result<TFr, u8>,
    compute_blob_kzg_proof: &dyn Fn(&[TFr], &TG1, &TKZGSettings) -> TG1,
    verify_blob_kzg_proof: &dyn Fn(&[TFr], &TG1, &TG1, &TKZGSettings) -> bool,
) {
    set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
    let ts = load_trusted_setup(TRUSTED_SETUP_PATH);
    let mut rng = rand::thread_rng();

    // Some preparation
    let blob: Vec<TFr> = generate_random_blob_bytes(&mut rng)
        .chunks(BYTES_PER_FIELD_ELEMENT)
        .map(|chunk| {
            bytes_to_bls_field(
                chunk
                    .try_into()
                    .expect("Chunked into incorrect number of bytes"),
            )
            .unwrap()
        })
        .collect();

    // Compute the proof
    let commitment = blob_to_kzg_commitment(&blob, &ts);
    let proof = compute_blob_kzg_proof(&blob, &commitment, &ts);

    // Finally verify the proof
    let result = verify_blob_kzg_proof(&blob, &commitment, &proof, &ts);
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
    load_trusted_setup: &dyn Fn(&str) -> TKZGSettings,
    blob_to_kzg_commitment: &dyn Fn(&[TFr], &TKZGSettings) -> TG1,
    bytes_to_bls_field: &dyn Fn(&[u8; 32usize]) -> Result<TFr, u8>,
    compute_blob_kzg_proof: &dyn Fn(&[TFr], &TG1, &TKZGSettings) -> TG1,
    verify_blob_kzg_proof: &dyn Fn(&[TFr], &TG1, &TG1, &TKZGSettings) -> bool,
) {
    set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
    let ts = load_trusted_setup(TRUSTED_SETUP_PATH);
    let mut rng = rand::thread_rng();

    // Some preparation
    let blob: Vec<TFr> = generate_random_blob_bytes(&mut rng)
        .chunks(BYTES_PER_FIELD_ELEMENT)
        .map(|chunk| {
            bytes_to_bls_field(
                chunk
                    .try_into()
                    .expect("Chunked into incorrect number of bytes"),
            )
            .unwrap()
        })
        .collect();

    // Compute the proof
    let commitment = blob_to_kzg_commitment(&blob, &ts);
    let mut proof = compute_blob_kzg_proof(&blob, &commitment, &ts);

    // Change the proof so it should not verify
    proof = proof.add(&TG1::generator());

    // Finally verify the proof
    let result = verify_blob_kzg_proof(&blob, &commitment, &proof, &ts);
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
    load_trusted_setup: &dyn Fn(&str) -> TKZGSettings,
    blob_to_kzg_commitment: &dyn Fn(&[TFr], &TKZGSettings) -> TG1,
    bytes_to_bls_field: &dyn Fn(&[u8; 32usize]) -> Result<TFr, u8>,
    compute_blob_kzg_proof: &dyn Fn(&[TFr], &TG1, &TKZGSettings) -> TG1,
    verify_blob_kzg_proof_batch: &dyn Fn(&[Vec<TFr>], &[TG1], &[TG1], &TKZGSettings) -> bool,
) {
    set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
    let ts = load_trusted_setup(TRUSTED_SETUP_PATH);
    let mut rng = rand::thread_rng();

    const N_SAMPLES: usize = 16;

    let mut blobs: Vec<Vec<TFr>> = Vec::new();
    let mut commitments: Vec<TG1> = Vec::new();
    let mut proofs: Vec<TG1> = Vec::new();

    // Some preparation
    for _ in 0..N_SAMPLES {
        let blob: Vec<TFr> = generate_random_blob_bytes(&mut rng)
            .chunks(BYTES_PER_FIELD_ELEMENT)
            .map(|chunk| {
                bytes_to_bls_field(
                    chunk
                        .try_into()
                        .expect("Chunked into incorrect number of bytes"),
                )
                .unwrap()
            })
            .collect();

        let commitment = blob_to_kzg_commitment(&blob, &ts);
        let proof = compute_blob_kzg_proof(&blob, &commitment, &ts);

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
        );
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
    load_trusted_setup: &dyn Fn(&str) -> TKZGSettings,
    blob_to_kzg_commitment: &dyn Fn(&[TFr], &TKZGSettings) -> TG1,
    bytes_to_bls_field: &dyn Fn(&[u8; 32usize]) -> Result<TFr, u8>,
    compute_blob_kzg_proof: &dyn Fn(&[TFr], &TG1, &TKZGSettings) -> TG1,
    verify_blob_kzg_proof_batch: &dyn Fn(&[Vec<TFr>], &[TG1], &[TG1], &TKZGSettings) -> bool,
) {
    set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
    let ts = load_trusted_setup(TRUSTED_SETUP_PATH);
    let mut rng = rand::thread_rng();

    const N_SAMPLES: usize = 2;

    let mut blobs: Vec<Vec<TFr>> = Vec::new();
    let mut commitments: Vec<TG1> = Vec::new();
    let mut proofs: Vec<TG1> = Vec::new();

    // Some preparation
    for _ in 0..N_SAMPLES {
        let blob: Vec<TFr> = generate_random_blob_bytes(&mut rng)
            .chunks(BYTES_PER_FIELD_ELEMENT)
            .map(|chunk| {
                bytes_to_bls_field(
                    chunk
                        .try_into()
                        .expect("Chunked into incorrect number of bytes"),
                )
                .unwrap()
            })
            .collect();

        let commitment = blob_to_kzg_commitment(&blob, &ts);
        let proof = compute_blob_kzg_proof(&blob, &commitment, &ts);

        blobs.push(blob);
        commitments.push(commitment);
        proofs.push(proof);
    }

    // Overwrite second proof with an incorrect one
    proofs[1] = proofs[0].clone();

    let result = verify_blob_kzg_proof_batch(&blobs, &commitments, &proofs, &ts);
    assert!(!result);
}
