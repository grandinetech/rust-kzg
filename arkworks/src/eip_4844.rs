use ark_poly::DenseUVPolynomial;
use kzg::FFTG1;
use kzg::common_utils::reverse_bit_order;
use crate::fft_g1::g1_linear_combination;
use crate::kzg_proofs::{pairings_verify, FFTSettings, KZGSettings, UniPoly_381, KZG};
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
use kzg::{cfg_into_iter, FFTSettings as FFTSettingsT, Fr, G1Mul, KZGSettings as LKZGSettings, G2};
use kzg::{Poly, G1};
use std::fs::File;
use std::io::Read;
use ark_ec::CurveGroup;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

pub fn hash_to_bls_field(x: &[u8; BYTES_PER_FIELD_ELEMENT]) -> ArkFr {
    ArkFr::from_bytes(x).unwrap()
}

#[allow(clippy::useless_conversion)]
pub fn bytes_to_blob(bytes: &[u8]) -> Result<Vec<ArkFr>, String> {
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
                .and_then(ArkFr::from_bytes)
        })
        .collect()
}

fn is_trusted_setup_in_lagrange_form(g1_values: &Vec<ArkG1>, g2_values: &Vec<ArkG2>) -> bool {
    if g1_values.len() < 2 || g2_values.len() < 2 {
        return false;
    }

    let is_monotomial_form =
        pairings_verify(&g1_values[1], &g2_values[0], &g1_values[0], &g2_values[1]);

    !is_monotomial_form
}

fn load_trusted_setup_rust(g1_bytes: &[u8], g2_bytes: &[u8]) -> Result<KZGSettings, String> {
    let num_g1_points = g1_bytes.len() / BYTES_PER_G1;
    if num_g1_points != FIELD_ELEMENTS_PER_BLOB {
        return Err(String::from("Invalid number of G1 points"));
    }

    let num_g2_points = g2_bytes.len() / BYTES_PER_G2;
    if num_g2_points != TRUSTED_SETUP_NUM_G2_POINTS {
        return Err(String::from("Invalid number of G2 points"));
    }

    let mut g1_values = g1_bytes
        .chunks(BYTES_PER_G1)
        .map(ArkG1::from_bytes)
        .collect::<Result<Vec<ArkG1>, String>>()?;

    let g2_values = g2_bytes
        .chunks(BYTES_PER_G2)
        .map(ArkG2::from_bytes)
        .collect::<Result<Vec<ArkG2>, String>>()?;

    if !is_trusted_setup_in_lagrange_form(&g1_values, &g2_values) {
        return Err(String::from("Trusted setup is not in Lagrange form"));
    }

    let mut max_scale: usize = 0;
    while (1 << max_scale) < num_g1_points {
        max_scale += 1;
    }

    let fs = FFTSettings::new(max_scale).unwrap();
    reverse_bit_order(&mut g1_values)?;

    Ok(KZGSettings {
        secret_g1: g1_values,
        secret_g2: g2_values,
        fs,
    })
}

pub fn load_trusted_setup(filepath: &str) -> Result<KZGSettings, String> {
    let mut file = File::open(filepath).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    let (g1_bytes, g2_bytes) = load_trusted_setup_string(&contents)?;
    load_trusted_setup_rust(g1_bytes.as_slice(), g2_bytes.as_slice())
}

fn fr_batch_inv(out: &mut [ArkFr], a: &[ArkFr], len: usize) {
    assert!(len > 0);

    let mut accumulator = ArkFr::one();

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

pub fn g1_lincomb(points: &[ArkG1], scalars: &[ArkFr], length: usize) -> ArkG1 {
    let mut out = ArkG1::default();
    g1_linear_combination(&mut out, points, scalars, length);
    out
}

pub fn compute_powers(base: &ArkFr, num_powers: usize) -> Vec<ArkFr> {
    let mut powers: Vec<ArkFr> = vec![ArkFr::default(); num_powers];
    if num_powers == 0 {
        return powers;
    }
    powers[0] = ArkFr::one();
    for i in 1..num_powers {
        powers[i] = powers[i - 1].mul(base);
    }
    powers
}

pub fn blob_to_kzg_commitment(blob: &[ArkFr], ks: &KZGSettings) -> Result<ArkG1, String> {
    let p = blob_to_polynomial(blob)?;
    poly_to_kzg_commitment(&p, ks)
}

pub fn verify_kzg_proof(
    commitment: &ArkG1,
    z: &ArkFr,
    y: &ArkFr,
    proof: &ArkG1,
    ks: &KZGSettings,
) -> Result<bool, String> {
    if !commitment.is_valid() {
        return Err("Invalid commitment".to_string());
    }
    if !proof.is_valid() {
        return Err("Invalid proof".to_string());
    }

    Ok(ks
        .check_proof_single(commitment, proof, z, y)
        .unwrap_or(false))
}

pub fn verify_kzg_proof_batch(
    commitments_g1: &[ArkG1],
    zs_fr: &[ArkFr],
    ys_fr: &[ArkFr],
    proofs_g1: &[ArkG1],
    ks: &KZGSettings,
) -> bool {
    let n = commitments_g1.len();
    let mut c_minus_y: Vec<ArkG1> = Vec::new();
    let mut r_times_z: Vec<ArkFr> = Vec::new();

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

pub fn compute_kzg_proof(blob: &[ArkFr], z: &ArkFr, ks: &KZGSettings) -> Result<(ArkG1, ArkFr),String> {
    let polynomial = blob_to_polynomial(blob)?;
    let y = evaluate_polynomial_in_evaluation_form(&polynomial, z, ks)?;
    let proof = ks.compute_proof_single(&polynomial, z)?;
    Ok((proof, y))
}

pub fn evaluate_polynomial_in_evaluation_form(p: &PolyData, x: &ArkFr, ks: &KZGSettings) -> Result<ArkFr, String> {
    assert_eq!(p.coeffs.len(), FIELD_ELEMENTS_PER_BLOB);

    let roots_of_unity: &Vec<ArkFr> = &ks.fs.roots_of_unity;
    let mut inverses_in: Vec<ArkFr> = vec![ArkFr::default(); FIELD_ELEMENTS_PER_BLOB];
    let mut inverses: Vec<ArkFr> = vec![ArkFr::default(); FIELD_ELEMENTS_PER_BLOB];

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        if x.equals(&roots_of_unity[i]) {
            return Ok(p.get_coeff_at(i));
        }
        inverses_in[i] = x.sub(&roots_of_unity[i]);
    }

    fr_batch_inv(&mut inverses, &inverses_in, FIELD_ELEMENTS_PER_BLOB);

    let mut tmp: ArkFr;
    let mut out = ArkFr::zero();

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        out.fr += (inverses[i].fr * (&roots_of_unity[i].fr)) * &p.coeffs[i].fr;
    }

    tmp = ArkFr::from_u64(FIELD_ELEMENTS_PER_BLOB as u64);
    out.fr /= &tmp.fr;
    tmp = x.pow(FIELD_ELEMENTS_PER_BLOB);
    tmp = tmp.sub(&ArkFr::one());
    out = out.mul(&tmp);
    Ok(out)
}

fn compute_challenge(blob: &[ArkFr], commitment: &ArkG1) -> ArkFr {
    let mut bytes: Vec<u8> = vec![0; CHALLENGE_INPUT_SIZE];

    // Copy domain separator
    bytes[..16].copy_from_slice(&FIAT_SHAMIR_PROTOCOL_DOMAIN);
    bytes_of_uint64(&mut bytes[16..24], FIELD_ELEMENTS_PER_BLOB as u64);
    // Set all other bytes of this 16-byte (little-endian) field to zero
    bytes_of_uint64(&mut bytes[24..32], 0);

    // Copy blob
    for i in 0..blob.len() {
        let v = blob[i].to_bytes();
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
    commitments_g1: &[ArkG1],
    zs_fr: &[ArkFr],
    ys_fr: &[ArkFr],
    proofs_g1: &[ArkG1],
) -> Vec<ArkFr> {
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
        let v = zs_fr[i].to_bytes();
        bytes[offset..(v.len() + offset)].copy_from_slice(&v[..]);
        offset += BYTES_PER_FIELD_ELEMENT;

        // Copy polynomial's evaluation value
        let v = ys_fr[i].to_bytes();
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

pub fn blob_to_polynomial(blob: &[ArkFr]) -> Result<PolyData, String> {
    assert_eq!(blob.len(), FIELD_ELEMENTS_PER_BLOB);
    let mut p: PolyData = PolyData::new(FIELD_ELEMENTS_PER_BLOB);
    p.coeffs = blob.to_vec();
    Ok(p)
}

fn poly_to_kzg_commitment(p: &PolyData, ks: &KZGSettings) -> Result<ArkG1, String> {
    assert_eq!(p.coeffs.len(), FIELD_ELEMENTS_PER_BLOB);
    Ok(g1_lincomb(&ks.secret_g1, &p.coeffs, FIELD_ELEMENTS_PER_BLOB))
}

pub fn compute_blob_kzg_proof(
    blob: &[ArkFr],
    commitment: &ArkG1,
    ks: &KZGSettings,
) -> Result<ArkG1, String> {
    if !commitment.is_valid() {
        return Err("Invalid commitment".to_string());
    }

    let evaluation_challenge_fr = compute_challenge(blob, commitment);
    let (proof, _) = compute_kzg_proof(blob, &evaluation_challenge_fr, ks)?;
    Ok(proof)
}

pub fn verify_blob_kzg_proof(
    blob: &[ArkFr],
    commitment_g1: &ArkG1,
    proof_g1: &ArkG1,
    ks: &KZGSettings,
) -> Result<bool, String> {
    if !commitment_g1.is_valid() {
        return Err("Invalid commitment".to_string());
    }
    if !proof_g1.is_valid() {
        return Err("Invalid proof".to_string());
    }

    let polynomial = blob_to_polynomial(blob)?;
    let evaluation_challenge_fr = compute_challenge(blob, commitment_g1);
    let y_fr = evaluate_polynomial_in_evaluation_form(&polynomial, &evaluation_challenge_fr, ks)?;
    verify_kzg_proof(commitment_g1, &evaluation_challenge_fr, &y_fr, proof_g1, ks)
}

fn compute_challenges_and_evaluate_polynomial(
    blobs: &[Vec<ArkFr>],
    commitments_g1: &[ArkG1],
    ks: &KZGSettings,
) -> Result<(Vec<ArkFr>, Vec<ArkFr>), String> {
    let mut evaluation_challenges_fr = Vec::new();
    let mut ys_fr = Vec::new();

    for i in 0..blobs.len() {
        let polynomial = blob_to_polynomial(&blobs[i])?;
        let evaluation_challenge_fr = compute_challenge(&blobs[i], &commitments_g1[i]);
        let y_fr =
            evaluate_polynomial_in_evaluation_form(&polynomial, &evaluation_challenge_fr, ks)?;

        evaluation_challenges_fr.push(evaluation_challenge_fr);
        ys_fr.push(y_fr);
    }

    Ok((evaluation_challenges_fr, ys_fr))
}

fn validate_batched_input(commitments: &[ArkG1], proofs: &[ArkG1]) -> Result<(), String> {
    let invalid_commitment = cfg_into_iter!(commitments).any(|&commitment| !commitment.is_valid());
    let invalid_proof = cfg_into_iter!(proofs).any(|&proof| !proof.is_valid());

    if invalid_commitment {
        return Err("Invalid commitment".to_string());
    }
    if invalid_proof {
        return Err("Invalid proof".to_string());
    }

    Ok(())
}

pub fn verify_blob_kzg_proof_batch(
    blobs: &[Vec<ArkFr>],
    commitments_g1: &[ArkG1],
    proofs_g1: &[ArkG1],
    ks: &KZGSettings,
) -> Result<bool, String> {
    // Exit early if we are given zero blobs
    if blobs.is_empty() {
        return Ok(true);
    }

    // For a single blob, just do a regular single verification
    if blobs.len() == 1 {
        return verify_blob_kzg_proof(&blobs[0], &commitments_g1[0], &proofs_g1[0], ks);
    }

    if blobs.len() != commitments_g1.len() || blobs.len() != proofs_g1.len() {
        return Err("Invalid amount of arguments".to_string());
    }

    #[cfg(feature = "parallel")]
    {
        let num_blobs = blobs.len();
        let num_cores = num_cpus::get_physical();

        return if num_blobs > num_cores {
            validate_batched_input(commitments_g1, proofs_g1)?;

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
                            ks,
                        ).unwrap();

                    verify_kzg_proof_batch(
                        commitment_group,
                        &evaluation_challenges_fr,
                        &ys_fr,
                        proof_group,
                        ks,
                    )
                }))
        } else {
            // Each group contains either one or zero blobs, so iterate
            // over the single blob verification function in parallel
            Ok((blobs, commitments_g1, proofs_g1).into_par_iter().all(
                |(blob, commitment, proof)| {
                    verify_blob_kzg_proof(blob, commitment, proof, ks).unwrap()
                },
            ))
        };
    }

    #[cfg(not(feature = "parallel"))]
    {
        validate_batched_input(commitments_g1, proofs_g1)?;
        let (evaluation_challenges_fr, ys_fr) =
            compute_challenges_and_evaluate_polynomial(blobs, commitments_g1, ks)?;

        Ok(verify_kzg_proof_batch(
            commitments_g1,
            &evaluation_challenges_fr,
            &ys_fr,
            proofs_g1,
            ks,
        ))
    }
}
