use kzg::{FFTSettings, Fr, KZGSettings, Poly, G1, G2};
use rand::rngs::ThreadRng;
use rand::Rng;
use std::env::set_current_dir;

fn u64_to_bytes(x: u64) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[0..8].copy_from_slice(&x.to_le_bytes());
    bytes
}

fn bytes32_from_hex(hex: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[(i * 2)..(i * 2 + 2)], 16).expect("invalid hex string");
    }
    out
}

fn bytes48_from_hex(hex: &str) -> [u8; 48] {
    let mut out = [0u8; 48];
    for (i, byte) in out.iter_mut().enumerate() {
        let byte_str = &hex[(i * 2)..(i * 2 + 2)];
        *byte = u8::from_str_radix(byte_str, 16).expect("invalid hex string");
    }
    out
}

const FIELD_ELEMENTS_PER_BLOB: usize = 4096;
const BYTES_PER_BLOB: usize = 32 * FIELD_ELEMENTS_PER_BLOB;
const BYTES_PER_FIELD_ELEMENT: usize = 32;

fn generate_random_blob_raw(rng: &mut ThreadRng) -> [u8; BYTES_PER_BLOB] {
    let mut arr = [0u8; BYTES_PER_BLOB];
    rng.fill(&mut arr[..]);
    // Ensure that the blob is canonical by ensuring that
    // each field element contained in the blob is < BLS_MODULUS
    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        arr[i * BYTES_PER_FIELD_ELEMENT + BYTES_PER_FIELD_ELEMENT - 1] = 0;
    }
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
    let x_bls = bytes_to_bls_field(&x_bytes);

    assert_eq!(bytes_from_bls_field(&x_bls), x_bytes);
    assert_eq!(x, x_bls.to_u64_arr()[0]);
}

pub fn compute_powers_test<TFr: Fr>(
    bytes_to_bls_field: &dyn Fn(&[u8; 32usize]) -> TFr,
    compute_powers: &dyn Fn(&TFr, usize) -> Vec<TFr>,
) {
    let x: u64 = 32930439;
    let n = 11;

    let x_bytes: [u8; 32] = u64_to_bytes(x);
    let x_bls = bytes_to_bls_field(&x_bytes);
    let powers = compute_powers(&x_bls, n);

    for (p, expected_p) in powers.iter().zip(EXPECTED_POWERS.iter()) {
        assert_eq!(expected_p, &p.to_u64_arr());
    }
}

pub fn blob_to_kzg_commitment_test<
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
    bytes_to_g1: &dyn Fn(&[u8; 48usize]) -> Result<TG1, String>,
) {
    set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
    let ts = load_trusted_setup("src/trusted_setups/trusted_setup.txt");

    let field_element_bytes =
        bytes32_from_hex("ad5570f5a3810b7af9d4b24bc1c2ea670245db2eaa49aae654b8f7393a9a6214");
    let field_element = bytes_to_bls_field(&field_element_bytes).unwrap();

    // Initialize the blob with a single field element
    let blob: [TFr; 1] = [field_element];

    // Get a commitment to this particular blob
    let commitment = blob_to_kzg_commitment(&blob, &ts);

    // We expect the commitment to match
    // If it doesn't match, something important has changed
    let expected_commitment_bytes = bytes48_from_hex(
        "9815ded2101b6d233fdf31d826ba0557778506df8526f42a\
        87ccd82db36a238b50f8965c25d4484782097436d29e458e",
    );
    let expected_commitment = bytes_to_g1(&expected_commitment_bytes).unwrap();
    assert!(commitment.equals(&expected_commitment));
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
    compute_blob_kzg_proof: &dyn Fn(&[TFr], &TKZGSettings) -> TG1,
    verify_blob_kzg_proof: &dyn Fn(&[TFr], &TG1, &TG1, &TKZGSettings) -> bool,
) {
    set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
    let ts = load_trusted_setup("src/trusted_setups/trusted_setup.txt");

    // Some preparation
    let mut rng = rand::thread_rng();
    let blob: Vec<TFr> = generate_random_blob_raw(&mut rng)
        .chunks(32)
        .map(|x| {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(x);
            bytes_to_bls_field(&bytes).unwrap()
        })
        .collect();

    // Compute the proof
    let commitment = blob_to_kzg_commitment(&blob, &ts);
    let proof = compute_blob_kzg_proof(&blob, &ts);

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
    compute_blob_kzg_proof: &dyn Fn(&[TFr], &TKZGSettings) -> TG1,
    verify_blob_kzg_proof: &dyn Fn(&[TFr], &TG1, &TG1, &TKZGSettings) -> bool,
) {
    set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
    let ts = load_trusted_setup("src/trusted_setups/trusted_setup.txt");

    // Some preparation
    let mut rng = rand::thread_rng();
    let blob: Vec<TFr> = generate_random_blob_raw(&mut rng)
        .chunks(32)
        .map(|x| {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(x);
            bytes_to_bls_field(&bytes).unwrap()
        })
        .collect();

    // Compute the proof
    let commitment = blob_to_kzg_commitment(&blob, &ts);
    let mut proof = compute_blob_kzg_proof(&blob, &ts);

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
    compute_blob_kzg_proof: &dyn Fn(&[TFr], &TKZGSettings) -> TG1,
    verify_blob_kzg_proof_batch: &dyn Fn(&[Vec<TFr>], &[TG1], &[TG1], &TKZGSettings) -> bool,
) {
    const N_SAMPLES: usize = 16;

    let mut blobs: Vec<Vec<TFr>> = Vec::new();
    let mut commitments: Vec<TG1> = Vec::new();
    let mut proofs: Vec<TG1> = Vec::new();

    set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
    let ts = load_trusted_setup("src/trusted_setups/trusted_setup.txt");

    // Some preparation
    for _ in 0..N_SAMPLES {
        let mut rng = rand::thread_rng();
        let blob: Vec<TFr> = generate_random_blob_raw(&mut rng)
            .chunks(32)
            .map(|x| {
                let mut bytes = [0u8; 32];
                bytes.copy_from_slice(x);
                bytes_to_bls_field(&bytes).unwrap()
            })
            .collect();

        let commitment = blob_to_kzg_commitment(&blob, &ts);
        let proof = compute_blob_kzg_proof(&blob, &ts);

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
    compute_blob_kzg_proof: &dyn Fn(&[TFr], &TKZGSettings) -> TG1,
    verify_blob_kzg_proof_batch: &dyn Fn(&[Vec<TFr>], &[TG1], &[TG1], &TKZGSettings) -> bool,
) {
    const N_SAMPLES: usize = 2;

    let mut blobs: Vec<Vec<TFr>> = Vec::new();
    let mut commitments: Vec<TG1> = Vec::new();
    let mut proofs: Vec<TG1> = Vec::new();

    set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
    let ts = load_trusted_setup("src/trusted_setups/trusted_setup.txt");

    // Some preparation
    for _ in 0..N_SAMPLES {
        let mut rng = rand::thread_rng();
        let blob: Vec<TFr> = generate_random_blob_raw(&mut rng)
            .chunks(32)
            .map(|x| {
                let mut bytes = [0u8; 32];
                bytes.copy_from_slice(x);
                bytes_to_bls_field(&bytes).unwrap()
            })
            .collect();

        let commitment = blob_to_kzg_commitment(&blob, &ts);
        let proof = compute_blob_kzg_proof(&blob, &ts);

        blobs.push(blob);
        commitments.push(commitment);
        proofs.push(proof);
    }

    // Overwrite second proof with an incorrect one
    proofs[1] = proofs[0].clone();

    let result = verify_blob_kzg_proof_batch(&blobs, &commitments, &proofs, &ts);
    assert!(!result);
}
