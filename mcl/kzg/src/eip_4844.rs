use crate::data_types::g1::g1_linear_combination;
use crate::data_types::{fr::*, g1::G1, g2::G2};
use crate::fk20_fft::*;
use crate::kzg10::{Curve, Polynomial};
use crate::kzg_settings::KZGSettings;
use crate::utilities::reverse_bit_order;
use kzg::eip_4844::{
    bytes_of_uint64, hash, load_trusted_setup_string, BYTES_PER_BLOB, BYTES_PER_COMMITMENT,
    BYTES_PER_FIELD_ELEMENT, BYTES_PER_G1, BYTES_PER_G2, BYTES_PER_PROOF, CHALLENGE_INPUT_SIZE,
    FIAT_SHAMIR_PROTOCOL_DOMAIN, FIELD_ELEMENTS_PER_BLOB, RANDOM_CHALLENGE_KZG_BATCH_DOMAIN,
    TRUSTED_SETUP_NUM_G2_POINTS,
};
use kzg::{cfg_into_iter, G1 as _, G2 as _};
use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use std::usize;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

pub fn hash_to_bls_field(x: &[u8; BYTES_PER_FIELD_ELEMENT]) -> Fr {
    Fr::from_bytes(x).unwrap()
}

#[allow(clippy::useless_conversion)]
pub fn bytes_to_blob(bytes: &[u8]) -> Result<Vec<Fr>, String> {
    if bytes.len() != BYTES_PER_BLOB {
        return Err(format!(
            "Invalid byte length. Expected {} got {}",
            BYTES_PER_BLOB,
            bytes.len(),
        ));
    }

    bytes
        .chunks(BYTES_PER_FIELD_ELEMENT)
        .map(|chunk| {
            chunk
                .try_into()
                .map_err(|_| "Chunked into incorrect number of bytes".to_string())
                .and_then(Fr::from_bytes)
        })
        .collect()
}

#[allow(clippy::useless_conversion)]
pub fn load_trusted_setup_from_bytes(g1_bytes: &[u8], g2_bytes: &[u8]) -> KZGSettings {
    let num_g1_points = g1_bytes.len() / BYTES_PER_G1;

    assert_eq!(num_g1_points, FIELD_ELEMENTS_PER_BLOB);
    assert_eq!(g2_bytes.len() / BYTES_PER_G2, TRUSTED_SETUP_NUM_G2_POINTS);

    let g1_projectives: Vec<G1> = g1_bytes
        .chunks(BYTES_PER_G1)
        .map(|chunk| {
            G1::from_bytes(
                chunk
                    .try_into()
                    .expect("Chunked into incorrect number of bytes"),
            )
            .unwrap()
        })
        .collect();

    let g2_values: Vec<G2> = g2_bytes
        .chunks(BYTES_PER_G2)
        .map(|chunk| {
            G2::from_bytes(
                chunk
                    .try_into()
                    .expect("Chunked into incorrect number of bytes"),
            )
            .unwrap()
        })
        .collect();

    let mut max_scale: usize = 0;
    while (1 << max_scale) < num_g1_points {
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

    let (g1_bytes, g2_bytes) = load_trusted_setup_string(&contents);
    load_trusted_setup_from_bytes(g1_bytes.as_slice(), g2_bytes.as_slice())
}

fn g1_lincomb(points: &[G1], scalars: &[Fr], length: usize) -> G1 {
    let mut out = G1::default();
    g1_linear_combination(&mut out, points, scalars, length);
    out
}

pub fn blob_to_kzg_commitment(blob: &[Fr], s: &KZGSettings) -> G1 {
    let p = blob_to_polynomial(blob);
    poly_to_kzg_commitment(&p, s)
}

pub fn verify_kzg_proof(
    commitment: &G1,
    z: &Fr,
    y: &Fr,
    proof: &G1,
    ks: &KZGSettings,
) -> Result<bool, String> {
    if !commitment.is_valid() {
        return Err("Invalid commitment".to_string());
    }
    if !proof.is_valid() {
        return Err("Invalid proof".to_string());
    }

    Ok(ks.curve.is_proof_valid(commitment, proof, z, y))
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
    let roots_of_unity: &Vec<Fr> = &s.fft_settings.roots_of_unity;

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

    let roots_of_unity: &Vec<Fr> = &s.fft_settings.roots_of_unity;
    let mut inverses_in = vec![Fr::default(); FIELD_ELEMENTS_PER_BLOB];
    let mut inverses = vec![Fr::default(); FIELD_ELEMENTS_PER_BLOB];

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

pub fn compute_blob_kzg_proof(blob: &[Fr], commitment: &G1, s: &KZGSettings) -> Result<G1, String> {
    if !commitment.is_valid() {
        return Err("Invalid commitment".to_string());
    }

    let evaluation_challenge_fr = compute_challenge(blob, commitment);
    let (proof, _) = compute_kzg_proof(blob, &evaluation_challenge_fr, s);
    Ok(proof)
}

pub fn verify_blob_kzg_proof(
    blob: &[Fr],
    commitment_g1: &G1,
    proof_g1: &G1,
    ts: &KZGSettings,
) -> Result<bool, String> {
    if !commitment_g1.is_valid() {
        return Err("Invalid commitment".to_string());
    }
    if !proof_g1.is_valid() {
        return Err("Invalid proof".to_string());
    }

    let polynomial = blob_to_polynomial(blob);
    let evaluation_challenge_fr = compute_challenge(blob, commitment_g1);
    let y_fr = evaluate_polynomial_in_evaluation_form(&polynomial, &evaluation_challenge_fr, ts);
    verify_kzg_proof(commitment_g1, &evaluation_challenge_fr, &y_fr, proof_g1, ts)
}

fn compute_challenges_and_evaluate_polynomial(
    blobs: &[Vec<Fr>],
    commitments_g1: &[G1],
    ts: &KZGSettings,
) -> (Vec<Fr>, Vec<Fr>) {
    let mut evaluation_challenges_fr = Vec::new();
    let mut ys_fr = Vec::new();

    for i in 0..blobs.len() {
        let polynomial = blob_to_polynomial(&blobs[i]);
        let evaluation_challenge_fr = compute_challenge(&blobs[i], &commitments_g1[i]);
        let y_fr =
            evaluate_polynomial_in_evaluation_form(&polynomial, &evaluation_challenge_fr, ts);

        evaluation_challenges_fr.push(evaluation_challenge_fr);
        ys_fr.push(y_fr);
    }

    (evaluation_challenges_fr, ys_fr)
}

pub fn verify_blob_kzg_proof_batch(
    blobs: &[Vec<Fr>],
    commitments_g1: &[G1],
    proofs_g1: &[G1],
    ts: &KZGSettings,
) -> Result<bool, String> {
    // Exit early if we are given zero blobs
    if blobs.is_empty() {
        return Ok(true);
    }

    // For a single blob, just do a regular single verification
    if blobs.len() == 1 {
        return verify_blob_kzg_proof(&blobs[0], &commitments_g1[0], &proofs_g1[0], ts);
    }

    if blobs.len() != commitments_g1.len() || blobs.len() != proofs_g1.len() {
        return Err("Invalid amount of arguments".to_string());
    }

    let invalid_commitment =
        cfg_into_iter!(commitments_g1).any(|&commitment| !commitment.is_valid());

    let invalid_proof = cfg_into_iter!(proofs_g1).any(|&proof| !proof.is_valid());

    if invalid_commitment {
        return Err("Invalid commitment".to_string());
    }

    if invalid_proof {
        return Err("Invalid proof".to_string());
    }

    #[cfg(feature = "parallel")]
    {
        let num_blobs = blobs.len();
        let num_cores = num_cpus::get_physical();

        return if num_blobs > num_cores {
            // Process blobs in parallel subgroups
            let blobs_per_group = num_blobs / num_cores;

            Ok(blobs
                .par_chunks(blobs_per_group)
                .enumerate()
                .all(|(i, blob_group)| {
                    let num_blobs_in_group = blob_group.len();
                    let commitment_group = &commitments_g1
                        [blobs_per_group * i..blobs_per_group * i + num_blobs_in_group];
                    let proof_group =
                        &proofs_g1[blobs_per_group * i..blobs_per_group * i + num_blobs_in_group];
                    let (evaluation_challenges_fr, ys_fr) =
                        compute_challenges_and_evaluate_polynomial(
                            blob_group,
                            commitment_group,
                            ts,
                        );

                    verify_kzg_proof_batch(
                        commitment_group,
                        &evaluation_challenges_fr,
                        &ys_fr,
                        proof_group,
                        ts,
                    )
                }))
        } else {
            // Each group contains either one or zero blobs, so iterate
            // over the single blob verification function in parallel
            Ok((blobs, commitments_g1, proofs_g1).into_par_iter().all(
                |(blob, commitment, proof)| {
                    verify_blob_kzg_proof(blob, commitment, proof, ts).unwrap()
                },
            ))
        };
    }

    #[cfg(not(feature = "parallel"))]
    {
        let (evaluation_challenges_fr, ys_fr) =
            compute_challenges_and_evaluate_polynomial(blobs, commitments_g1, ts);

        Ok(verify_kzg_proof_batch(
            commitments_g1,
            &evaluation_challenges_fr,
            &ys_fr,
            proofs_g1,
            ts,
        ))
    }
}

fn compute_challenge(blob: &[Fr], commitment: &G1) -> Fr {
    let mut bytes: Vec<u8> = vec![0; CHALLENGE_INPUT_SIZE];

    // Copy domain separator
    bytes[..16].copy_from_slice(&FIAT_SHAMIR_PROTOCOL_DOMAIN);
    bytes_of_uint64(&mut bytes[16..24], FIELD_ELEMENTS_PER_BLOB as u64);
    // Set all other bytes of this 16-byte (little-endian) field to zero
    bytes_of_uint64(&mut bytes[24..32], 0);

    // Copy blob
    for i in 0..blob.len() {
        let v = Fr::to_bytes(&blob[i]);
        bytes[(32 + i * BYTES_PER_FIELD_ELEMENT)..(32 + (i + 1) * BYTES_PER_FIELD_ELEMENT)]
            .copy_from_slice(&v);
    }

    // Copy commitment
    let v = commitment.to_bytes();
    for i in 0..v.len() {
        bytes[32 + BYTES_PER_BLOB + i] = v[i];
    }

    // Now let's create the challenge!
    let eval_challenge = hash(&bytes);
    hash_to_bls_field(&eval_challenge)
}

fn compute_r_powers(
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
        let v = commitments_g1[i].to_bytes();
        bytes[offset..(v.len() + offset)].copy_from_slice(&v[..]);
        offset += BYTES_PER_COMMITMENT;

        // Copy evaluation challenge
        let v = Fr::to_bytes(&zs_fr[i]);
        bytes[offset..(v.len() + offset)].copy_from_slice(&v[..]);
        offset += BYTES_PER_FIELD_ELEMENT;

        // Copy polynomial's evaluation value
        let v = Fr::to_bytes(&ys_fr[i]);
        bytes[offset..(v.len() + offset)].copy_from_slice(&v[..]);
        offset += BYTES_PER_FIELD_ELEMENT;

        // Copy proof
        let v = proofs_g1[i].to_bytes();
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
