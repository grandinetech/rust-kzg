use crate::fft_g1::g1_linear_combination;
use crate::kzg_proofs::{pairings_verify, FFTSettings, KZGSettings, UniPoly_381, KZG};
use crate::kzg_types::{ArkG1, ArkG2, FsFr};
use crate::utils::{pc_g1projective_into_blst_p1, pc_g2projective_into_blst_p2, PolyData};
use ark_bls12_381::Bls12_381;
use ark_ec::ProjectiveCurve;
use ark_std::test_rng;
use kzg::eip_4844::{
    bytes32_from_hex, bytes48_from_hex, bytes_of_uint64, hash, load_trusted_setup_string,
    BYTES_PER_BLOB, BYTES_PER_COMMITMENT, BYTES_PER_FIELD_ELEMENT, BYTES_PER_G1, BYTES_PER_G2,
    BYTES_PER_PROOF, CHALLENGE_INPUT_SIZE, FIAT_SHAMIR_PROTOCOL_DOMAIN, FIELD_ELEMENTS_PER_BLOB,
    RANDOM_CHALLENGE_KZG_BATCH_DOMAIN, TRUSTED_SETUP_NUM_G2_POINTS,
};
use kzg::{FFTSettings as FFTSettingsT, Fr, G1Mul, KZGSettings as LKZGSettings, G2};
use kzg::{Poly, G1};
use std::fs::File;
use std::io::Read;

pub fn bytes_to_bls_field(bytes: &[u8; BYTES_PER_FIELD_ELEMENT]) -> Result<FsFr, u8> {
    FsFr::from_bytes(*bytes)
}

pub fn bytes_from_bls_field(fr: &FsFr) -> [u8; BYTES_PER_FIELD_ELEMENT] {
    fr.to_bytes()
}

pub fn bytes_to_g1(bytes: &[u8; BYTES_PER_G1]) -> Result<ArkG1, String> {
    Ok(ArkG1::from_bytes(*bytes))
}

pub fn bytes_from_g1(g1: &ArkG1) -> [u8; BYTES_PER_G1] {
    g1.to_bytes()
}

pub fn hex_to_bls_field(hex: &str) -> FsFr {
    let fr_bytes = bytes32_from_hex(hex);
    bytes_to_bls_field(&fr_bytes).unwrap()
}

pub fn hex_to_g1(hex: &str) -> ArkG1 {
    let g1_bytes = bytes48_from_hex(hex);
    bytes_to_g1(&g1_bytes).unwrap()
}

pub fn hash_to_bls_field(x: &[u8; BYTES_PER_FIELD_ELEMENT]) -> FsFr {
    bytes_to_bls_field(x).unwrap()
}

fn load_trusted_setup_rust(g1_bytes: &[u8], g2_bytes: &[u8]) -> KZGSettings {
    let num_g1_points = g1_bytes.len() / BYTES_PER_G1;

    assert_eq!(num_g1_points, FIELD_ELEMENTS_PER_BLOB);
    assert_eq!(g2_bytes.len() / BYTES_PER_G2, TRUSTED_SETUP_NUM_G2_POINTS);

    let mut max_scale: usize = 0;
    while (1 << max_scale) < num_g1_points {
        max_scale += 1;
    }

    let fs = FFTSettings::new(max_scale).unwrap();

    let length = num_g1_points + 1;
    let rng = &mut test_rng();
    let mut setup = KZG::<Bls12_381, UniPoly_381>::setup(length, false, rng).unwrap();

    let mut temp = Vec::new();
    let mut temp2 = Vec::new();
    let mut temp3 = Vec::new();

    for i in 0..length {
        temp.push(pc_g1projective_into_blst_p1(setup.g1_secret[i as usize]).unwrap());
        temp2.push(pc_g2projective_into_blst_p2(setup.g2_secret[i as usize]).unwrap());
        temp3.push(setup.g1_secret[i as usize].into_affine());
    }

    setup.params.powers_of_g = temp3;

    KZGSettings {
        fs,
        secret_g1: temp,
        secret_g2: temp2,
        length: length as u64,
        params: setup.params,
        ..KZGSettings::default()
    }
}

pub fn load_trusted_setup(filepath: &str) -> KZGSettings {
    let mut file = File::open(filepath).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    let (g1_bytes, g2_bytes) = load_trusted_setup_string(&contents);
    load_trusted_setup_rust(g1_bytes.as_slice(), g2_bytes.as_slice())
}

fn fr_batch_inv(out: &mut [FsFr], a: &[FsFr], len: usize) {
    assert!(len > 0);

    let mut accumulator = FsFr::one();

    for i in 0..len {
        out[i] = accumulator;
        accumulator = accumulator.mul(&a[i]);
    }

    accumulator = accumulator.inverse();

    for i in (0..len).rev() {
        out[i] = out[i].mul(&accumulator);
        accumulator = accumulator.mul(&a[i]);
    }
}

pub fn g1_lincomb(points: &[ArkG1], scalars: &[FsFr], length: usize) -> ArkG1 {
    let mut out = ArkG1::default();
    g1_linear_combination(&mut out, points, scalars, length);
    out
}

pub fn compute_powers(base: &FsFr, num_powers: usize) -> Vec<FsFr> {
    let mut powers: Vec<FsFr> = vec![FsFr::default(); num_powers];
    if num_powers == 0 {
        return powers;
    }
    powers[0] = FsFr::one();
    for i in 1..num_powers {
        powers[i] = powers[i - 1].mul(base);
    }
    powers
}

pub fn blob_to_kzg_commitment(blob: &[FsFr], ks: &KZGSettings) -> ArkG1 {
    let p = blob_to_polynomial(blob);
    poly_to_kzg_commitment(&p, ks)
}

pub fn verify_kzg_proof(
    commitment: &ArkG1,
    z: &FsFr,
    y: &FsFr,
    proof: &ArkG1,
    ks: &KZGSettings,
) -> bool {
    ks.check_proof_single(commitment, proof, z, y)
        .unwrap_or(false)
}

pub fn verify_kzg_proof_batch(
    commitments_g1: &[ArkG1],
    zs_fr: &[FsFr],
    ys_fr: &[FsFr],
    proofs_g1: &[ArkG1],
    ks: &KZGSettings,
) -> bool {
    let n = commitments_g1.len();
    let mut c_minus_y: Vec<ArkG1> = Vec::new();
    let mut r_times_z: Vec<FsFr> = Vec::new();

    // Compute the random lincomb challenges
    let r_powers = compute_r_powers(commitments_g1, zs_fr, ys_fr, proofs_g1);

    // Compute \sum r^i * Proof_i
    let proof_lincomb = g1_lincomb(proofs_g1, &r_powers, n);

    for i in 0..n {
        // Get [y_i]
        let ys_encrypted = ArkG1::generator().mul(&ys_fr[i]);
        // Get C_i - [y_i]
        c_minus_y.push(commitments_g1[i].sub(&ys_encrypted));
        // Get r^i * z_i
        r_times_z.push(r_powers[i].mul(&zs_fr[i]));
    }

    // Get \sum r^i z_i Proof_i
    let proof_z_lincomb = g1_lincomb(proofs_g1, &r_times_z, n);
    // Get \sum r^i (C_i - [y_i])
    let mut c_minus_y_lincomb = g1_lincomb(&c_minus_y, &r_powers, n);

    // Get C_minus_y_lincomb + proof_z_lincomb
    let rhs_g1 = c_minus_y_lincomb.add_or_dbl(&proof_z_lincomb);

    // Do the pairing check!
    pairings_verify(
        &proof_lincomb,
        &ks.secret_g2[1],
        &rhs_g1,
        &ArkG2::generator(),
    )
}

pub fn compute_kzg_proof(blob: &[FsFr], z: &FsFr, ks: &KZGSettings) -> (ArkG1, FsFr) {
    let polynomial = blob_to_polynomial(blob);
    let y = evaluate_polynomial_in_evaluation_form(&polynomial, z, ks);
    let proof = ks.compute_proof_single(&polynomial, z).unwrap();
    (proof, y)
}

pub fn evaluate_polynomial_in_evaluation_form(p: &PolyData, x: &FsFr, ks: &KZGSettings) -> FsFr {
    assert_eq!(p.coeffs.len(), FIELD_ELEMENTS_PER_BLOB);

    let roots_of_unity: &Vec<FsFr> = &ks.fs.roots_of_unity;
    let mut inverses_in: Vec<FsFr> = vec![FsFr::default(); FIELD_ELEMENTS_PER_BLOB];
    let mut inverses: Vec<FsFr> = vec![FsFr::default(); FIELD_ELEMENTS_PER_BLOB];

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        if x.equals(&roots_of_unity[i]) {
            return p.get_coeff_at(i);
        }
        inverses_in[i] = x.sub(&roots_of_unity[i]);
    }

    fr_batch_inv(&mut inverses, &inverses_in, FIELD_ELEMENTS_PER_BLOB);

    let mut tmp: FsFr;
    let mut out = FsFr::zero();

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        tmp = inverses[i].mul(&roots_of_unity[i]);
        tmp = tmp.mul(&p.coeffs[i]);
        out = out.add(&tmp);
    }

    tmp = FsFr::from_u64(FIELD_ELEMENTS_PER_BLOB as u64);
    out = out.div(&tmp).unwrap();
    tmp = x.pow(FIELD_ELEMENTS_PER_BLOB);
    tmp = tmp.sub(&FsFr::one());
    out = out.mul(&tmp);
    out
}

fn compute_challenge(blob: &[FsFr], commitment: &ArkG1) -> FsFr {
    let mut bytes: Vec<u8> = vec![0; CHALLENGE_INPUT_SIZE];

    // Copy domain separator
    bytes[..16].copy_from_slice(&FIAT_SHAMIR_PROTOCOL_DOMAIN);
    bytes_of_uint64(&mut bytes[16..24], FIELD_ELEMENTS_PER_BLOB as u64);
    // Set all other bytes of this 16-byte (little-endian) field to zero
    bytes_of_uint64(&mut bytes[24..32], 0);

    // Copy blob
    for i in 0..blob.len() {
        let v = bytes_from_bls_field(&blob[i]);
        bytes[(32 + i * BYTES_PER_FIELD_ELEMENT)..(32 + (i + 1) * BYTES_PER_FIELD_ELEMENT)]
            .copy_from_slice(&v);
    }

    // Copy commitment
    let v = bytes_from_g1(commitment);
    for i in 0..v.len() {
        bytes[32 + BYTES_PER_BLOB + i] = v[i];
    }

    // Now let's create the challenge!
    let eval_challenge = hash(&bytes);
    hash_to_bls_field(&eval_challenge)
}

fn compute_r_powers(
    commitments_g1: &[ArkG1],
    zs_fr: &[FsFr],
    ys_fr: &[FsFr],
    proofs_g1: &[ArkG1],
) -> Vec<FsFr> {
    let n = commitments_g1.len();
    let input_size =
        32 + n * (BYTES_PER_COMMITMENT + 2 * BYTES_PER_FIELD_ELEMENT + BYTES_PER_PROOF);

    #[allow(unused_assignments)]
    let mut offset = 0;
    let mut bytes: Vec<u8> = vec![0; input_size];

    // Copy domain separator
    bytes[..16].copy_from_slice(&RANDOM_CHALLENGE_KZG_BATCH_DOMAIN);
    bytes_of_uint64(&mut bytes[16..24], FIELD_ELEMENTS_PER_BLOB as u64);
    bytes_of_uint64(&mut bytes[24..32], n as u64);
    offset = 32;

    for i in 0..n {
        // Copy commitment
        let v = bytes_from_g1(&commitments_g1[i]);
        bytes[offset..(v.len() + offset)].copy_from_slice(&v[..]);
        offset += BYTES_PER_COMMITMENT;

        // Copy evaluation challenge
        let v = bytes_from_bls_field(&zs_fr[i]);
        bytes[offset..(v.len() + offset)].copy_from_slice(&v[..]);
        offset += BYTES_PER_FIELD_ELEMENT;

        // Copy polynomial's evaluation value
        let v = bytes_from_bls_field(&ys_fr[i]);
        bytes[offset..(v.len() + offset)].copy_from_slice(&v[..]);
        offset += BYTES_PER_FIELD_ELEMENT;

        // Copy proof
        let v = bytes_from_g1(&proofs_g1[i]);
        bytes[offset..(v.len() + offset)].copy_from_slice(&v[..]);
        offset += BYTES_PER_PROOF;
    }

    // Make sure we wrote the entire buffer
    assert_eq!(offset, input_size);

    // Now let's create the challenge!
    let eval_challenge = hash(&bytes);
    let r = hash_to_bls_field(&eval_challenge);
    compute_powers(&r, n)
}

pub fn blob_to_polynomial(blob: &[FsFr]) -> PolyData {
    assert_eq!(blob.len(), FIELD_ELEMENTS_PER_BLOB);
    let mut p: PolyData = PolyData::new(FIELD_ELEMENTS_PER_BLOB).unwrap();
    p.coeffs = blob.to_vec();
    p
}

fn poly_to_kzg_commitment(p: &PolyData, ks: &KZGSettings) -> ArkG1 {
    assert_eq!(p.coeffs.len(), FIELD_ELEMENTS_PER_BLOB);
    g1_lincomb(&ks.secret_g1, &p.coeffs, FIELD_ELEMENTS_PER_BLOB)
}

pub fn compute_blob_kzg_proof(blob: &[FsFr], commitment: &ArkG1, ks: &KZGSettings) -> ArkG1 {
    let evaluation_challenge_fr = compute_challenge(blob, commitment);
    let (proof, _) = compute_kzg_proof(blob, &evaluation_challenge_fr, ks);
    proof
}

pub fn verify_blob_kzg_proof(
    blob: &[FsFr],
    commitment_g1: &ArkG1,
    proof_g1: &ArkG1,
    ks: &KZGSettings,
) -> bool {
    let polynomial = blob_to_polynomial(blob);
    let evaluation_challenge_fr = compute_challenge(blob, commitment_g1);
    let y_fr = evaluate_polynomial_in_evaluation_form(&polynomial, &evaluation_challenge_fr, ks);
    verify_kzg_proof(commitment_g1, &evaluation_challenge_fr, &y_fr, proof_g1, ks)
}

pub fn verify_blob_kzg_proof_batch(
    blobs: &[Vec<FsFr>],
    commitments_g1: &[ArkG1],
    proofs_g1: &[ArkG1],
    ks: &KZGSettings,
) -> bool {
    // Exit early if we are given zero blobs
    if blobs.is_empty() {
        return true;
    }

    // For a single blob, just do a regular single verification
    if blobs.len() == 1 {
        return verify_blob_kzg_proof(&blobs[0], &commitments_g1[0], &proofs_g1[0], ks);
    }

    let mut evaluation_challenges_fr: Vec<FsFr> = Vec::new();
    let mut ys_fr: Vec<FsFr> = Vec::new();

    for i in 0..blobs.len() {
        let polynomial = blob_to_polynomial(&blobs[i]);
        let evaluation_challenge_fr = compute_challenge(&blobs[i], &commitments_g1[i]);
        let y_fr =
            evaluate_polynomial_in_evaluation_form(&polynomial, &evaluation_challenge_fr, ks);

        evaluation_challenges_fr.push(evaluation_challenge_fr);
        ys_fr.push(y_fr);
    }

    verify_kzg_proof_batch(
        commitments_g1,
        &evaluation_challenges_fr,
        &ys_fr,
        proofs_g1,
        ks,
    )
}
