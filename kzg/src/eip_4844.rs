#![allow(non_camel_case_types)]
use alloc::string::String;
use alloc::vec;
pub use blst::{blst_fr, blst_p1, blst_p2};
use core::ffi::c_uint;
use sha2::{Digest, Sha256};

use crate::common_utils::reverse_bit_order;
use crate::{Fr, G1, G1Mul, G2, FFTSettings, Poly, KZGSettings, PairingVerify};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

////////////////////////////// Constant values for EIP-4844 //////////////////////////////

pub const FIELD_ELEMENTS_PER_BLOB: usize = if cfg!(feature = "minimal-spec") {
    4
} else {
    4096
};

pub const BYTES_PER_G1: usize = 48;
pub const BYTES_PER_G2: usize = 96;
pub const BYTES_PER_BLOB: usize = BYTES_PER_FIELD_ELEMENT * FIELD_ELEMENTS_PER_BLOB;
pub const BYTES_PER_FIELD_ELEMENT: usize = 32;
pub const BYTES_PER_PROOF: usize = 48;
pub const BYTES_PER_COMMITMENT: usize = 48;

pub const TRUSTED_SETUP_PATH: &str = if cfg!(feature = "minimal-spec") {
    "src/trusted_setups/trusted_setup_4.txt"
} else {
    "src/trusted_setups/trusted_setup.txt"
};

// Currently, we only support fixed amount of G1 and G2 points contained in trusted setups.
// Issue arises when a binding using the C API loads different G1 point quantities each time.
pub static mut TRUSTED_SETUP_NUM_G1_POINTS: usize = 0;

pub const TRUSTED_SETUP_NUM_G2_POINTS: usize = 65;

pub const CHALLENGE_INPUT_SIZE: usize =
    FIAT_SHAMIR_PROTOCOL_DOMAIN.len() + 16 + BYTES_PER_BLOB + BYTES_PER_COMMITMENT;

pub const FIAT_SHAMIR_PROTOCOL_DOMAIN: [u8; 16] = [
    70, 83, 66, 76, 79, 66, 86, 69, 82, 73, 70, 89, 95, 86, 49, 95,
]; // "FSBLOBVERIFY_V1_"

pub const RANDOM_CHALLENGE_KZG_BATCH_DOMAIN: [u8; 16] = [
    82, 67, 75, 90, 71, 66, 65, 84, 67, 72, 95, 95, 95, 86, 49, 95,
]; // "RCKZGBATCH___V1_"

////////////////////////////// C API for EIP-4844 //////////////////////////////

pub type C_KZG_RET = c_uint;

pub const C_KZG_RET_OK: C_KZG_RET = 0;
pub const C_KZG_RET_BADARGS: C_KZG_RET = 1;
pub const C_KZG_RET_ERROR: C_KZG_RET = 2;
pub const C_KZG_RET_MALLOC: C_KZG_RET = 3;

#[repr(C)]
pub struct Bytes32 {
    pub bytes: [u8; 32],
}

#[repr(C)]
pub struct Bytes48 {
    pub bytes: [u8; 48],
}

#[repr(C)]
pub struct BLSFieldElement {
    pub bytes: [u8; BYTES_PER_FIELD_ELEMENT],
}

#[repr(C)]
pub struct Blob {
    pub bytes: [u8; BYTES_PER_BLOB],
}

#[repr(C)]
pub struct KZGCommitment {
    pub bytes: [u8; BYTES_PER_COMMITMENT],
}

#[repr(C)]
pub struct KZGProof {
    pub bytes: [u8; BYTES_PER_PROOF],
}

#[repr(C)]
pub struct CKZGSettings {
    pub max_width: u64,
    pub roots_of_unity: *mut blst_fr,
    pub g1_values: *mut blst_p1,
    pub g2_values: *mut blst_p2,
}

////////////////////////////// Utility functions for EIP-4844 //////////////////////////////

pub fn load_trusted_setup_string(contents: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
    let mut offset = 0;

    const TRUSTED_SETUP_ERROR: &str = "Incorrect trusted setup format";

    #[inline(always)]
    fn scan_number(offset: &mut usize, contents: &str) -> Result<usize, String> {
        *offset += contents[(*offset)..]
            .find(|c: char| !c.is_whitespace())
            .ok_or_else(|| String::from(TRUSTED_SETUP_ERROR))?;
        let start = *offset;
        *offset += contents[(*offset)..]
            .find(|c: char| !c.is_ascii_digit())
            .ok_or_else(|| String::from(TRUSTED_SETUP_ERROR))?;
        let end = *offset;
        contents[start..end]
            .parse::<usize>()
            .map_err(|_| String::from(TRUSTED_SETUP_ERROR))
    }

    let g1_point_count = scan_number(&mut offset, contents)?;

    if g1_point_count != FIELD_ELEMENTS_PER_BLOB {
        return Err(String::from(TRUSTED_SETUP_ERROR));
    }

    let g2_point_count = scan_number(&mut offset, contents)?;

    if g2_point_count != TRUSTED_SETUP_NUM_G2_POINTS {
        return Err(String::from(TRUSTED_SETUP_ERROR));
    }

    let mut g1_bytes = vec![0u8; g1_point_count * BYTES_PER_G1];
    let mut g2_bytes = vec![0u8; g2_point_count * BYTES_PER_G2];

    #[inline(always)]
    fn scan_hex_byte(offset: &mut usize, contents: &str) -> Result<u8, String> {
        *offset += contents[(*offset)..]
            .find(|c: char| !c.is_whitespace())
            .ok_or_else(|| String::from(TRUSTED_SETUP_ERROR))?;
        let start = *offset;

        let end = if contents
            .get((*offset + 1)..)
            .map(|it| {
                it.chars()
                    .next()
                    .map(|c| c.is_ascii_hexdigit())
                    .unwrap_or(false)
            })
            .unwrap_or(false)
        {
            *offset += 2;
            *offset
        } else {
            *offset += 1;
            *offset
        };

        u8::from_str_radix(&contents[start..end], 16).map_err(|_| String::from(TRUSTED_SETUP_ERROR))
    }

    for byte in &mut g1_bytes {
        *byte = scan_hex_byte(&mut offset, contents)?
    }

    for byte in &mut g2_bytes {
        *byte = scan_hex_byte(&mut offset, contents)?
    }

    Ok((g1_bytes, g2_bytes))
}

pub fn bytes_of_uint64(out: &mut [u8], mut n: u64) {
    for byte in out.iter_mut().rev().take(8) {
        *byte = (n & 0xff) as u8;
        n >>= 8;
    }
}

pub fn hash(x: &[u8]) -> [u8; 32] {
    Sha256::digest(x).into()
}

#[macro_export]
macro_rules! cfg_into_iter {
    ($e: expr) => {{
        #[cfg(feature = "parallel")]
        let result = $e.into_par_iter();

        #[cfg(not(feature = "parallel"))]
        let result = $e.into_iter();

        result
    }};
}

////////////////////////////// Trait based implementations of functions for EIP-4844 //////////////////////////////

fn poly_to_kzg_commitment<TFr: Fr, TG1: G1 + G1Mul<TFr>, TG2: G2, TFFTSettings: FFTSettings<TFr>, TPoly: Poly<TFr>, TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>>(p: &TPoly, s: &TKZGSettings) -> TG1 {
    TG1::g1_lincomb(s.get_g1_secret(), p.get_coeffs(), FIELD_ELEMENTS_PER_BLOB)
}

pub fn blob_to_kzg_commitment_rust<TFr: Fr, TG1: G1 + G1Mul<TFr>, TG2: G2, TFFTSettings: FFTSettings<TFr>, TPoly: Poly<TFr>, TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>> (
    blob: &[TFr],
    settings: &TKZGSettings,
) -> Result<TG1, String> {
    let polynomial = blob_to_polynomial(blob)?;

    Ok(poly_to_kzg_commitment(&polynomial, settings))
}

pub fn compute_powers<TFr: Fr>(base: &TFr, num_powers: usize) -> Vec<TFr> {
    let mut powers: Vec<TFr> = vec![TFr::default(); num_powers];
    if num_powers == 0 {
        return powers;
    }
    powers[0] = TFr::one();
    for i in 1..num_powers {
        powers[i] = powers[i - 1].mul(base);
    }
    powers
}


fn compute_r_powers<TG1: G1, TFr: Fr>(
    commitments_g1: &[TG1],
    zs_fr: &[TFr],
    ys_fr: &[TFr],
    proofs_g1: &[TG1],
) -> Result<Vec<TFr>, String> {
    let n = commitments_g1.len();
    let input_size =
        32 + n * (BYTES_PER_COMMITMENT + 2 * BYTES_PER_FIELD_ELEMENT + BYTES_PER_PROOF);

    let mut bytes: Vec<u8> = vec![0; input_size];

    // Copy domain separator
    bytes[..16].copy_from_slice(&RANDOM_CHALLENGE_KZG_BATCH_DOMAIN);
    bytes_of_uint64(&mut bytes[16..24], FIELD_ELEMENTS_PER_BLOB as u64);
    bytes_of_uint64(&mut bytes[24..32], n as u64);
    let mut offset = 32;

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
    if offset != input_size {
        return Err(String::from("Error while copying commitments"));
    }

    // Now let's create the challenge!
    let eval_challenge = hash(&bytes);
    let r = hash_to_bls_field(&eval_challenge);

    Ok(compute_powers(&r, n))
}


fn verify_kzg_proof_batch<TFr: Fr, TG1: G1 + G1Mul<TFr> + PairingVerify<TG1, TG2>, TG2: G2, TFFTSettings: FFTSettings<TFr>, TPoly: Poly<TFr>, TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>>(
    commitments_g1: &[TG1],
    zs_fr: &[TFr],
    ys_fr: &[TFr],
    proofs_g1: &[TG1],
    ts: &TKZGSettings,
) -> Result<bool, String> {
    let n = commitments_g1.len();
    let mut c_minus_y: Vec<TG1> = Vec::with_capacity(n);
    let mut r_times_z: Vec<TFr> = Vec::with_capacity(n);

    // Compute the random lincomb challenges
    let r_powers = compute_r_powers(commitments_g1, zs_fr, ys_fr, proofs_g1)?;

    // Compute \sum r^i * Proof_i
    let proof_lincomb = TG1::g1_lincomb(proofs_g1, &r_powers, n);

    for i in 0..n {
        // Get [y_i]
        let ys_encrypted = TG1::generator().mul(&ys_fr[i]);
        // Get C_i - [y_i]
        c_minus_y.push(commitments_g1[i].sub(&ys_encrypted));
        // Get r^i * z_i
        r_times_z.push(r_powers[i].mul(&zs_fr[i]));
    }

    // Get \sum r^i z_i Proof_i
    let proof_z_lincomb = TG1::g1_lincomb(proofs_g1, &r_times_z, n);
    // Get \sum r^i (C_i - [y_i])
    let mut c_minus_y_lincomb = TG1::g1_lincomb(&c_minus_y, &r_powers, n);

    // Get C_minus_y_lincomb + proof_z_lincomb
    let rhs_g1 = c_minus_y_lincomb.add_or_dbl(&proof_z_lincomb);

    // Do the pairing check!
    Ok(TG1::verify(
        &proof_lincomb,
        &ts.get_g2_secret()[1],
        &rhs_g1,
        &TG2::generator(),
    ))
}

pub fn compute_kzg_proof_rust<TFr: Fr + Copy, TG1: G1 + G1Mul<TFr>, TG2: G2, TFFTSettings: FFTSettings<TFr>, TPoly: Poly<TFr>, TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>>(
    blob: &[TFr],
    z: &TFr,
    s: &TKZGSettings,
) -> Result<(TG1, TFr), String> {
    let polynomial = blob_to_polynomial(blob)?;
    let y = evaluate_polynomial_in_evaluation_form(&polynomial, z, s)?;

    let mut tmp: TFr;

    let mut m: usize = 0;
    let mut q: TPoly = TPoly::new(FIELD_ELEMENTS_PER_BLOB);

    let mut inverses_in: Vec<TFr> = vec![TFr::default(); FIELD_ELEMENTS_PER_BLOB];
    let mut inverses: Vec<TFr> = vec![TFr::default(); FIELD_ELEMENTS_PER_BLOB];

    let roots_of_unity = s.get_fft_settings().get_roots_of_unity();
    let poly_coeffs = polynomial.get_coeffs();

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        if z.equals(&roots_of_unity[i]) {
            // We are asked to compute a KZG proof inside the domain
            m = i + 1;
            inverses_in[i] = TFr::one();
            continue;
        }
        // (p_i - y) / (ω_i - z)
        q.set_coeff_at(i, &poly_coeffs[i].sub(&y));
        inverses_in[i] = roots_of_unity[i].sub(z);
    }

    fr_batch_inv(&mut inverses, &inverses_in, FIELD_ELEMENTS_PER_BLOB)?;

    for (i, inverse) in inverses.iter().enumerate().take(FIELD_ELEMENTS_PER_BLOB) {
        q.set_coeff_at(i, &q.get_coeff_at(i).mul(inverse));
    }

    if m != 0 {
        // ω_{m-1} == z
        m -= 1;
        q.set_coeff_at(m, &TFr::zero());
        for i in 0..FIELD_ELEMENTS_PER_BLOB {
            if i == m {
                continue;
            }
            // Build denominator: z * (z - ω_i)
            tmp = z.sub(&roots_of_unity[i]);
            inverses_in[i] = tmp.mul(z);
        }

        fr_batch_inv(&mut inverses, &inverses_in, FIELD_ELEMENTS_PER_BLOB)?;

        for i in 0..FIELD_ELEMENTS_PER_BLOB {
            if i == m {
                continue;
            }
            // Build numerator: ω_i * (p_i - y)
            tmp = poly_coeffs[i].sub(&y);
            tmp = tmp.mul(&roots_of_unity[i]);
            // Do the division: (p_i - y) * ω_i / (z * (z - ω_i))
            tmp = tmp.mul(&inverses[i]);
            q.set_coeff_at(m, &q.get_coeff_at(m).add(&tmp))
        }
    }

    let proof = TG1::g1_lincomb(s.get_g1_secret(), q.get_coeffs(), FIELD_ELEMENTS_PER_BLOB);
    Ok((proof, y))
}

pub fn compute_blob_kzg_proof_rust<TFr: Fr + Copy, TG1: G1 + G1Mul<TFr>, TG2: G2, TFFTSettings: FFTSettings<TFr>, TPoly: Poly<TFr>, TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>>(
    blob: &[TFr],
    commitment: &TG1,
    ts: &TKZGSettings,
) -> Result<TG1, String> {
    if !commitment.is_inf() && !commitment.is_valid() {
        return Err("Invalid commitment".to_string());
    }

    let evaluation_challenge_fr = compute_challenge(blob, commitment);
    let (proof, _) = compute_kzg_proof_rust::<_,_,_,_,_,_>(blob, &evaluation_challenge_fr, ts)?;
    Ok(proof)
}

pub fn verify_kzg_proof_rust<TFr: Fr, TG1: G1, TG2: G2, TFFTSettings: FFTSettings<TFr>, TPoly: Poly<TFr>, TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>>(
    commitment: &TG1,
    z: &TFr,
    y: &TFr,
    proof: &TG1,
    s: &TKZGSettings,
) -> Result<bool, String> {
    if !commitment.is_inf() && !commitment.is_valid() {
        return Err("Invalid commitment".to_string());
    }
    if !proof.is_inf() && !proof.is_valid() {
        return Err("Invalid proof".to_string());
    }

    s.check_proof_single(commitment, proof, z, y)
}

pub fn verify_blob_kzg_proof_rust<TFr: Fr + Copy, TG1: G1, TG2: G2, TFFTSettings: FFTSettings<TFr>, TPoly: Poly<TFr>, TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>>(
    blob: &[TFr],
    commitment_g1: &TG1,
    proof_g1: &TG1,
    ts: &TKZGSettings,
) -> Result<bool, String> {
    if !commitment_g1.is_inf() && !commitment_g1.is_valid() {
        return Err("Invalid commitment".to_string());
    }
    if !proof_g1.is_inf() && !proof_g1.is_valid() {
        return Err("Invalid proof".to_string());
    }

    let polynomial = blob_to_polynomial(blob)?;
    let evaluation_challenge_fr = compute_challenge(blob, commitment_g1);
    let y_fr =
        evaluate_polynomial_in_evaluation_form(&polynomial, &evaluation_challenge_fr, ts)?;
    verify_kzg_proof_rust(commitment_g1, &evaluation_challenge_fr, &y_fr, proof_g1, ts)
}

fn compute_challenges_and_evaluate_polynomial<TFr: Fr + Copy, TG1: G1, TG2: G2, TFFTSettings: FFTSettings<TFr>, TPoly: Poly<TFr>, TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>>(
    blobs: &[Vec<TFr>],
    commitments_g1: &[TG1],
    ts: &TKZGSettings,
) -> Result<(Vec<TFr>, Vec<TFr>), String> {
    let mut evaluation_challenges_fr = Vec::with_capacity(blobs.len());
    let mut ys_fr = Vec::with_capacity(blobs.len());

    for i in 0..blobs.len() {
        let polynomial = blob_to_polynomial(&blobs[i])?;
        let evaluation_challenge_fr = compute_challenge(&blobs[i], &commitments_g1[i]);
        let y_fr =
            evaluate_polynomial_in_evaluation_form(&polynomial, &evaluation_challenge_fr, ts)?;

        evaluation_challenges_fr.push(evaluation_challenge_fr);
        ys_fr.push(y_fr);
    }

    Ok((evaluation_challenges_fr, ys_fr))
}

fn validate_batched_input<TG1: G1>(commitments: &[TG1], proofs: &[TG1]) -> Result<(), String> {
    let invalid_commitment = cfg_into_iter!(commitments).any(|commitment| !commitment.is_inf() && !commitment.is_valid());
    let invalid_proof = cfg_into_iter!(proofs).any(|proof| !proof.is_inf() && !proof.is_valid());

    if invalid_commitment {
        return Err("Invalid commitment".to_string());
    }
    if invalid_proof {
        return Err("Invalid proof".to_string());
    }

    Ok(())
}

pub fn verify_blob_kzg_proof_batch_rust<TFr: Fr + Copy, TG1: G1 + G1Mul<TFr> + PairingVerify<TG1, TG2> , TG2: G2, TFFTSettings: FFTSettings<TFr>, TPoly: Poly<TFr>, TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly> + Sync>(
    blobs: &[Vec<TFr>],
    commitments_g1: &[TG1],
    proofs_g1: &[TG1],
    ts: &TKZGSettings,
) -> Result<bool, String> {
    // Exit early if we are given zero blobs
    if blobs.is_empty() {
        return Ok(true);
    }

    // For a single blob, just do a regular single verification
    if blobs.len() == 1 {
        return verify_blob_kzg_proof_rust(&blobs[0], &commitments_g1[0], &proofs_g1[0], ts);
    }

    if blobs.len() != commitments_g1.len() || blobs.len() != proofs_g1.len() {
        return Err("Invalid amount of arguments".to_string());
    }

    #[cfg(feature = "parallel")]
    {
        let num_blobs = blobs.len();
        let num_cores = num_cpus::get_physical();

        if num_blobs > num_cores {
            validate_batched_input(commitments_g1, proofs_g1)?;

            // Process blobs in parallel subgroups
            let blobs_per_group = num_blobs / num_cores;

            blobs
                .par_chunks(blobs_per_group)
                .enumerate()
                .map(|(i, blob_group)| {
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
                        )?;

                    verify_kzg_proof_batch(
                        commitment_group,
                        &evaluation_challenges_fr,
                        &ys_fr,
                        proof_group,
                        ts,
                    )
                })
                .try_reduce(|| true, |a, b| Ok(a && b))
        } else {
            // Each group contains either one or zero blobs, so iterate
            // over the single blob verification function in parallel
            (blobs, commitments_g1, proofs_g1)
                .into_par_iter()
                .map(|(blob, commitment, proof)| {
                    verify_blob_kzg_proof_rust(blob, commitment, proof, ts)
                })
                .try_reduce(|| true, |a, b| Ok(a && b))
        }
    }

    #[cfg(not(feature = "parallel"))]
    {
        validate_batched_input(commitments_g1, proofs_g1)?;
        let (evaluation_challenges_fr, ys_fr) =
            compute_challenges_and_evaluate_polynomial(blobs, commitments_g1, ts)?;

        verify_kzg_proof_batch(
            commitments_g1,
            &evaluation_challenges_fr,
            &ys_fr,
            proofs_g1,
            ts,
        )
    }
}

#[allow(clippy::useless_conversion)]
pub fn bytes_to_blob<TFr: Fr>(bytes: &[u8]) -> Result<Vec<TFr>, String> {
    if bytes.len() != BYTES_PER_BLOB {
        return Err(format!(
            "Invalid byte length. Expected {} got {}",
            BYTES_PER_BLOB,
            bytes.len(),
        ));
    }

    bytes
        .chunks(BYTES_PER_FIELD_ELEMENT)
        .map(TFr::from_bytes)
        .collect()
}

fn fr_batch_inv<TFr: Fr + PartialEq + Copy>(out: &mut [TFr], a: &[TFr], len: usize) -> Result<(), String> {
    if len == 0 {
        return Err(String::from("Length is less than 0."));
    }

    if a == out {
        return Err(String::from("Destination is the same as source."));
    }

    let mut accumulator = TFr::one();

    for i in 0..len {
        out[i] = accumulator;
        accumulator = accumulator.mul(&a[i]);
    }

    if accumulator.is_zero() {
        return Err(String::from("Zero input"));
    }

    accumulator = accumulator.eucl_inverse();

    for i in (0..len).rev() {
        out[i] = out[i].mul(&accumulator);
        accumulator = accumulator.mul(&a[i]);
    }

    Ok(())
}

pub fn hash_to_bls_field<TFr: Fr>(x: &[u8; BYTES_PER_FIELD_ELEMENT]) -> TFr {
    TFr::from_bytes_unchecked(x).unwrap()
}

fn compute_challenge<TFr: Fr, TG1: G1>(blob: &[TFr], commitment: &TG1) -> TFr {
    let mut bytes: Vec<u8> = vec![0; CHALLENGE_INPUT_SIZE];

    // Copy domain separator
    bytes[..16].copy_from_slice(&FIAT_SHAMIR_PROTOCOL_DOMAIN);
    // Set all other bytes of this 16-byte (big-endian) field to zero
    bytes_of_uint64(&mut bytes[16..24], 0);
    bytes_of_uint64(&mut bytes[24..32], FIELD_ELEMENTS_PER_BLOB as u64);

    for (i, field) in blob.iter().enumerate() {
        let v = field.to_bytes();
        let size = (32 + i * BYTES_PER_FIELD_ELEMENT)..(32 + (i + 1) * BYTES_PER_FIELD_ELEMENT);

        bytes[size].copy_from_slice(&v);
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

pub fn blob_to_polynomial<TFr: Fr, TPoly: Poly<TFr>>(blob: &[TFr]) -> Result<TPoly, String> {
    if blob.len() != FIELD_ELEMENTS_PER_BLOB {
        return Err(String::from("Blob length must be FIELD_ELEMENTS_PER_BLOB"));
    }
    Ok(TPoly::from_coeffs(blob))
}

pub fn evaluate_polynomial_in_evaluation_form<TG1: G1, TG2: G2, TFr: Fr + Copy, TPoly: Poly<TFr>, TFFTSettings: FFTSettings<TFr>, TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>>(
    p: &TPoly,
    x: &TFr,
    s: &TKZGSettings,
) -> Result<TFr, String> {
    if p.len() != FIELD_ELEMENTS_PER_BLOB {
        return Err(String::from("Incorrect field elements count."));
    }

    let mut inverses_in: Vec<TFr> = vec![TFr::default(); FIELD_ELEMENTS_PER_BLOB];
    let mut inverses: Vec<TFr> = vec![TFr::default(); FIELD_ELEMENTS_PER_BLOB];

    let roots_of_unity = s.get_fft_settings().get_roots_of_unity();
    let poly_coeffs = p.get_coeffs();

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        if x == &roots_of_unity[i] {
            return Ok(poly_coeffs[i]);
        }
        inverses_in[i] = x.sub(&roots_of_unity[i]);
    }

    fr_batch_inv(&mut inverses, &inverses_in, FIELD_ELEMENTS_PER_BLOB)?;

    let mut tmp: TFr;
    let mut out = TFr::zero();

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        tmp = inverses[i].mul(&roots_of_unity[i]);
        tmp = tmp.mul(&poly_coeffs[i]);
        out = out.add(&tmp);
    }

    tmp = TFr::from_u64(FIELD_ELEMENTS_PER_BLOB as u64);
    out = match out.div(&tmp) {
        Ok(value) => value,
        Err(err) => return Err(err),
    };
    tmp = x.pow(FIELD_ELEMENTS_PER_BLOB);
    tmp = tmp.sub(&TFr::one());
    out = out.mul(&tmp);
    Ok(out)
}

fn is_trusted_setup_in_lagrange_form<TG1: G1 + PairingVerify<TG1, TG2>, TG2: G2>(g1_values: &Vec<TG1>, g2_values: &Vec<TG2>) -> bool {
    if g1_values.len() < 2 || g2_values.len() < 2 {
        return false;
    }

    let is_monotomial_form = TG1::verify(&g1_values[1], &g2_values[0], &g1_values[0], &g2_values[1]);
    !is_monotomial_form
}

#[allow(clippy::useless_conversion)]
pub fn load_trusted_setup_rust<TFr: Fr, TG1: G1 + PairingVerify<TG1, TG2>, TG2: G2, TFFTSettings: FFTSettings<TFr>, TPoly: Poly<TFr>, TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>>(g1_bytes: &[u8], g2_bytes: &[u8]) -> Result<TKZGSettings, String> {
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
        .map(TG1::from_bytes)
        .collect::<Result<Vec<TG1>, String>>()?;

    let g2_values = g2_bytes
        .chunks(BYTES_PER_G2)
        .map(TG2::from_bytes)
        .collect::<Result<Vec<TG2>, String>>()?;

    // Sanity check, that user is not trying to load old trusted setup file
    if !is_trusted_setup_in_lagrange_form::<TG1, TG2>(&g1_values, &g2_values) {
        return Err(String::from("Trusted setup is not in Lagrange form"));
    }

    let mut max_scale: usize = 0;
    while (1 << max_scale) < num_g1_points {
        max_scale += 1;
    }

    let fs = TFFTSettings::new(max_scale)?;
    reverse_bit_order(&mut g1_values)?;
    TKZGSettings::new(g1_values.as_slice(), g2_values.as_slice(), max_scale, &fs)
}