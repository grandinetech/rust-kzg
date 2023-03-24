use crate::data_types::g1::{g1_linear_combination, is_valid_order};
use crate::data_types::{fr::*, g1::G1, g2::G2};
use crate::fk20_fft::*;
use crate::kzg10::{Curve, Polynomial};
use crate::kzg_settings::KZGSettings;
use crate::mcl_methods::*;
use kzg::G1 as _;
use sha2::{Digest as _, Sha256};
use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use std::usize;

pub fn bytes_to_g1(bytes: &[u8; 48usize]) -> Result<G1, String> {
    set_eth_serialization(1);
    let mut g1 = G1::default();
    if !G1::deserialize(&mut g1, bytes) {
        return Err("failed to deserialize".to_string());
    }
    if !(g1.is_valid() && is_valid_order(&g1)) {
        return Err("the point is not in g1 group".to_string());
    }
    Ok(g1)
}

pub fn bytes_to_g2(bytes: &[u8; 96usize]) -> Result<G2, String> {
    set_eth_serialization(1);
    let mut g2 = G2::default();
    if !G2::deserialize(&mut g2, bytes) {
        return Err("failed to deserialize".to_string());
    }
    Ok(g2)
}

pub fn bytes_from_g1(g1: &G1) -> [u8; 48usize] {
    set_eth_serialization(1);
    G1::serialize(g1).try_into().unwrap()
}

pub fn load_trusted_setup_string(contents: &str) -> (Vec<u8>, Vec<u8>) {
    let mut lines = contents.lines();
    let length = lines.next().unwrap().parse::<usize>().unwrap();
    let n2 = lines.next().unwrap().parse::<usize>().unwrap();

    let g1_bytes = (0..length)
        .flat_map(|_| {
            let line = lines.next().unwrap();
            assert!(line.len() == 96);
            (0..line.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&line[i..i + 2], 16).unwrap())
                .collect::<Vec<u8>>()
        })
        .collect::<Vec<u8>>();

    let g2_bytes = (0..n2)
        .flat_map(|_| {
            let line = lines.next().unwrap();
            assert!(line.len() == 192);
            (0..line.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&line[i..i + 2], 16).unwrap())
                .collect::<Vec<u8>>()
        })
        .collect::<Vec<u8>>();

    (g1_bytes, g2_bytes)
}

pub fn load_trusted_setup_from_bytes(g1_bytes: &[u8], g2_bytes: &[u8]) -> KZGSettings {
    let g1_projectives = g1_bytes
        .chunks(48)
        .map(|chunk| {
            let mut bytes_array: [u8; 48] = [0; 48];
            bytes_array.copy_from_slice(chunk);
            bytes_to_g1(&bytes_array).unwrap()
        })
        .collect::<Vec<G1>>();

    let g2_values = g2_bytes
        .chunks(96)
        .map(|chunk| {
            let mut bytes_array: [u8; 96] = [0; 96];
            bytes_array.copy_from_slice(chunk);
            bytes_to_g2(&bytes_array).unwrap()
        })
        .collect::<Vec<G2>>();

    let length = g1_projectives.len();

    let mut max_scale: usize = 0;
    while (1 << max_scale) < length {
        max_scale += 1;
    }

    let fs = FFTSettings::new(max_scale as u8);
    let mut g1_values = fs.fft_g1_inv(&g1_projectives).unwrap();
    reverse_bit_order(&mut g1_values);

    KZGSettings {
        fft_settings: fs,
        curve: Curve {
            g1_gen: G1::gen(),
            g2_gen: G2::gen(),
            g1_points: g1_values,
            g2_points: g2_values,
        },
    }
}

pub fn load_trusted_setup(filepath: &str) -> KZGSettings {
    let mut file = File::open(filepath).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    let (b1, b2) = load_trusted_setup_string(&contents);
    load_trusted_setup_from_bytes(b1.as_slice(), b2.as_slice())
}

pub fn reverse_bit_order<T>(values: &mut [T])
where
    T: Clone,
{
    let unused_bit_len = values.len().leading_zeros() + 1;
    for i in 0..values.len() - 1 {
        let r = i.reverse_bits() >> unused_bit_len;
        if r > i {
            let tmp = values[r].clone();
            values[r] = values[i].clone();
            values[i] = tmp;
        }
    }
}

pub fn bytes_to_bls_field(bytes: &[u8; 32usize]) -> Result<Fr, u8> {
    Fr::from_scalar(bytes)
}

pub fn bytes_from_bls_field(fr: &Fr) -> [u8; 32usize] {
    Fr::to_scalar(fr)
}

pub fn g1_lincomb(points: &[G1], scalars: &[Fr], length: usize) -> G1 {
    let mut out = G1::default();
    g1_linear_combination(&mut out, points, scalars, length);
    out
}

pub fn blob_to_kzg_commitment(blob: &[Fr], s: &KZGSettings) -> G1 {
    let p = blob_to_polynomial(blob);
    poly_to_kzg_commitment(&p, s)
}

pub fn verify_kzg_proof(commitment: &G1, z: &Fr, y: &Fr, proof: &G1, ks: &KZGSettings) -> bool {
    ks.curve.is_proof_valid(commitment, proof, z, y)
}

pub fn verify_kzg_proof_batch(
    commitments_g1: &[G1],
    zs_fr: &[Fr],
    ys_fr: &[Fr],
    proofs_g1: &[G1],
    ts: &KZGSettings,
) -> bool {
    let n = commitments_g1.len();
    let mut c_minus_y: Vec<G1> = Vec::new();
    let mut r_times_z: Vec<Fr> = Vec::new();

    // Compute the random lincomb challenges
    let r_powers = compute_r_powers(commitments_g1, zs_fr, ys_fr, proofs_g1);

    // Compute \sum r^i * Proof_i
    let proof_lincomb = g1_lincomb(proofs_g1, &r_powers, n);

    for i in 0..n {
        // Get [y_i]
        let ys_encrypted = &G1::gen() * &ys_fr[i];
        // Get C_i - [y_i]
        c_minus_y.push(commitments_g1[i] - ys_encrypted);
        // Get r^i * z_i
        r_times_z.push(r_powers[i] * zs_fr[i]);
    }

    // Get \sum r^i z_i Proof_i
    let proof_z_lincomb = g1_lincomb(proofs_g1, &r_times_z, n);
    // Get \sum r^i (C_i - [y_i])
    let mut c_minus_y_lincomb = g1_lincomb(&c_minus_y, &r_powers, n);

    // Get C_minus_y_lincomb + proof_z_lincomb
    let rhs_g1 = c_minus_y_lincomb.add_or_dbl(&proof_z_lincomb);

    // Do the pairing check!
    Curve::verify_pairing(&proof_lincomb, &ts.curve.g2_points[1], &rhs_g1, &G2::gen())
}

pub fn compute_kzg_proof(blob: &[Fr], z: &Fr, s: &KZGSettings) -> (G1, Fr) {
    assert_eq!(blob.len(), FIELD_ELEMENTS_PER_BLOB);

    let polynomial = blob_to_polynomial(blob);
    let y = evaluate_polynomial_in_evaluation_form(&polynomial, z, s);

    let mut tmp: Fr;
    let mut roots_of_unity = s.fft_settings.exp_roots_of_unity.clone();
    reverse_bit_order(&mut roots_of_unity);

    let mut m = 0;
    let mut q = Polynomial::new(FIELD_ELEMENTS_PER_BLOB);

    let mut inverses_in: Vec<Fr> = vec![Fr::default(); FIELD_ELEMENTS_PER_BLOB];
    let mut inverses: Vec<Fr> = vec![Fr::default(); FIELD_ELEMENTS_PER_BLOB];

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        if z == &roots_of_unity[i] {
            // We are asked to compute a KZG proof inside the domain
            m = i + 1;
            inverses_in[i] = Fr::one();
            continue;
        }
        // (p_i - y) / (ω_i - z)
        q.coeffs[i] = polynomial.coeffs[i] - y;
        inverses_in[i] = &roots_of_unity[i] - z;
    }

    fr_batch_inv(&mut inverses, &inverses_in, FIELD_ELEMENTS_PER_BLOB);

    for (i, inverse) in inverses.iter().enumerate().take(FIELD_ELEMENTS_PER_BLOB) {
        q.coeffs[i] = &q.coeffs[i] * inverse;
    }

    if m != 0 {
        // ω_{m-1} == z
        m -= 1;
        q.coeffs[m] = Fr::zero();
        for i in 0..FIELD_ELEMENTS_PER_BLOB {
            if i == m {
                continue;
            }
            // Build denominator: z * (z - ω_i)
            tmp = z - &roots_of_unity[i];
            inverses_in[i] = &tmp * z;
        }

        fr_batch_inv(&mut inverses, &inverses_in, FIELD_ELEMENTS_PER_BLOB);

        for i in 0..FIELD_ELEMENTS_PER_BLOB {
            if i == m {
                continue;
            }
            // Build numerator: ω_i * (p_i - y)
            tmp = polynomial.coeffs[i] - y;
            tmp = tmp * roots_of_unity[i];
            // Do the division: (p_i - y) * ω_i / (z * (z - ω_i))
            tmp = tmp * inverses[i];
            q.coeffs[i] = q.coeffs[i] + tmp;
        }
    }

    let proof = g1_lincomb(
        &s.curve.g1_points,
        q.coeffs.as_slice(),
        FIELD_ELEMENTS_PER_BLOB,
    );
    (proof, y)
}

pub fn evaluate_polynomial_in_evaluation_form(p: &Polynomial, x: &Fr, s: &KZGSettings) -> Fr {
    assert_eq!(p.coeffs.len(), FIELD_ELEMENTS_PER_BLOB);

    let mut roots_of_unity = s.fft_settings.exp_roots_of_unity.clone();
    let mut inverses_in = vec![Fr::default(); FIELD_ELEMENTS_PER_BLOB];
    let mut inverses = vec![Fr::default(); FIELD_ELEMENTS_PER_BLOB];

    reverse_bit_order(&mut roots_of_unity);

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        if *x == roots_of_unity[i] {
            return p.coeffs[i];
        }
        inverses_in[i] = x - &roots_of_unity[i];
    }

    fr_batch_inv(
        inverses.as_mut_slice(),
        inverses_in.as_slice(),
        FIELD_ELEMENTS_PER_BLOB,
    );

    let mut tmp: Fr;
    let mut out = Fr::zero();

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        tmp = inverses[i] * roots_of_unity[i];
        tmp = tmp * p.coeffs[i];
        out = out + tmp;
    }

    let arr: [u64; 4] = [FIELD_ELEMENTS_PER_BLOB as u64, 0, 0, 0];
    tmp = Fr::from_u64_arr(&arr);
    out = out / tmp;
    tmp = x.pow(FIELD_ELEMENTS_PER_BLOB);
    tmp = tmp - Fr::one();
    out = out * tmp;
    out
}

pub fn compute_powers(base: &Fr, num_powers: usize) -> Vec<Fr> {
    let mut powers: Vec<Fr> = vec![Fr::default(); num_powers];
    if num_powers == 0 {
        return powers;
    }
    powers[0] = Fr::one();
    for i in 1..num_powers {
        powers[i] = &powers[i - 1] * base;
    }
    powers
}

fn bytes_of_uint64(out: &mut [u8], mut n: u64) {
    for byte in out.iter_mut().take(8) {
        *byte = (n & 0xff) as u8;
        n >>= 8;
    }
}

pub fn hash_to_bls_field(x: &[u8; 32]) -> Fr {
    Fr::from_scalar(x).unwrap()
}

fn fr_batch_inv(out: &mut [Fr], a: &[Fr], len: usize) {
    assert!(len > 0);

    let mut accumulator = Fr::one();

    for i in 0..len {
        out[i] = accumulator;
        accumulator = accumulator * a[i];
    }

    accumulator = accumulator.inverse();

    for i in (0..len).rev() {
        out[i] = out[i] * accumulator;
        accumulator = accumulator * a[i];
    }
}

pub fn compute_blob_kzg_proof(blob: &[Fr], commitment: &G1, s: &KZGSettings) -> G1 {
    let evaluation_challenge_fr = compute_challenge(blob, commitment);
    let (proof, _) = compute_kzg_proof(blob, &evaluation_challenge_fr, s);
    proof
}

pub fn verify_blob_kzg_proof(
    blob: &[Fr],
    commitment_g1: &G1,
    proof_g1: &G1,
    ts: &KZGSettings,
) -> bool {
    let polynomial = blob_to_polynomial(blob);
    let evaluation_challenge_fr = compute_challenge(blob, commitment_g1);
    let y_fr = evaluate_polynomial_in_evaluation_form(&polynomial, &evaluation_challenge_fr, ts);
    verify_kzg_proof(commitment_g1, &evaluation_challenge_fr, &y_fr, proof_g1, ts)
}

pub fn verify_blob_kzg_proof_batch(
    blobs: &[Vec<Fr>],
    commitments_g1: &[G1],
    proofs_g1: &[G1],
    ts: &KZGSettings,
) -> bool {
    // Exit early if we are given zero blobs
    if blobs.is_empty() {
        return true;
    }

    let mut evaluation_challenges_fr: Vec<Fr> = Vec::new();
    let mut ys_fr: Vec<Fr> = Vec::new();

    for i in 0..blobs.len() {
        let polynomial = blob_to_polynomial(&blobs[i]);
        let evaluation_challenge_fr = compute_challenge(&blobs[i], &commitments_g1[i]);
        let y_fr =
            evaluate_polynomial_in_evaluation_form(&polynomial, &evaluation_challenge_fr, ts);

        evaluation_challenges_fr.push(evaluation_challenge_fr);
        ys_fr.push(y_fr);
    }

    verify_kzg_proof_batch(
        commitments_g1,
        &evaluation_challenges_fr,
        &ys_fr,
        proofs_g1,
        ts,
    )
}

pub fn compute_challenge(blob: &[Fr], commitment: &G1) -> Fr {
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

pub fn compute_r_powers(
    commitments_g1: &[G1],
    zs_fr: &[Fr],
    ys_fr: &[Fr],
    proofs_g1: &[G1],
) -> Vec<Fr> {
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

pub fn blob_to_polynomial(blob: &[Fr]) -> Polynomial {
    assert_eq!(blob.len(), FIELD_ELEMENTS_PER_BLOB);
    let mut p = Polynomial::new(FIELD_ELEMENTS_PER_BLOB);
    p.coeffs = blob.to_vec();
    p
}

fn poly_to_kzg_commitment(p: &Polynomial, s: &KZGSettings) -> G1 {
    assert_eq!(p.coeffs.len(), FIELD_ELEMENTS_PER_BLOB);
    g1_lincomb(&s.curve.g1_points, &p.coeffs, FIELD_ELEMENTS_PER_BLOB)
}

fn hash(x: &[u8]) -> [u8; 32] {
    Sha256::digest(x).into()
}

pub const FIELD_ELEMENTS_PER_BLOB: usize = 4096;
pub const FIAT_SHAMIR_PROTOCOL_DOMAIN: [u8; 16] = [
    70, 83, 66, 76, 79, 66, 86, 69, 82, 73, 70, 89, 95, 86, 49, 95,
]; // "FSBLOBVERIFY_V1_"
pub const RANDOM_CHALLENGE_KZG_BATCH_DOMAIN: [u8; 16] = [
    82, 67, 75, 90, 71, 66, 65, 84, 67, 72, 95, 95, 95, 86, 49, 95,
]; // "RCKZGBATCH___V1_"
pub const BYTES_PER_FIELD_ELEMENT: usize = 32;
pub const BYTES_PER_BLOB: usize = 32 * FIELD_ELEMENTS_PER_BLOB;
pub const CHALLENGE_INPUT_SIZE: usize = 32 + BYTES_PER_BLOB + 48;
pub const BYTES_PER_COMMITMENT: usize = 48;
pub const BYTES_PER_PROOF: usize = 48;
