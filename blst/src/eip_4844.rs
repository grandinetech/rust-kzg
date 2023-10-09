extern crate alloc;

use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::ptr::null_mut;
#[cfg(feature = "std")]
use libc::FILE;
#[cfg(feature = "std")]
use std::fs::File;
#[cfg(feature = "std")]
use std::io::Read;

use blst::{blst_fr, blst_fr_from_scalar, blst_p1, blst_p2, blst_scalar, blst_scalar_from_bendian};
use kzg::{cfg_into_iter, FFTSettings, Fr, G1Mul, KZGSettings, Poly, G1, G2};

#[cfg(feature = "std")]
use kzg::eip_4844::load_trusted_setup_string;

use kzg::eip_4844::{
    bytes_of_uint64, hash, Blob, Bytes32, Bytes48, CKZGSettings, KZGCommitment, KZGProof,
    BYTES_PER_BLOB, BYTES_PER_COMMITMENT, BYTES_PER_FIELD_ELEMENT, BYTES_PER_G1, BYTES_PER_G2,
    BYTES_PER_PROOF, CHALLENGE_INPUT_SIZE, C_KZG_RET, C_KZG_RET_BADARGS, C_KZG_RET_OK,
    FIAT_SHAMIR_PROTOCOL_DOMAIN, FIELD_ELEMENTS_PER_BLOB, RANDOM_CHALLENGE_KZG_BATCH_DOMAIN,
    TRUSTED_SETUP_NUM_G1_POINTS, TRUSTED_SETUP_NUM_G2_POINTS,
};

use crate::types::fft_settings::FsFFTSettings;
use crate::types::fr::FsFr;
use crate::types::g1::FsG1;

use crate::kzg_proofs::{g1_linear_combination, pairings_verify};
use crate::types::g2::FsG2;
use crate::types::kzg_settings::FsKZGSettings;
use crate::types::poly::FsPoly;
use crate::utils::reverse_bit_order;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

pub fn hash_to_bls_field(x: &[u8; BYTES_PER_FIELD_ELEMENT]) -> FsFr {
    let mut tmp = blst_scalar::default();
    let mut out = blst_fr::default();
    unsafe {
        blst_scalar_from_bendian(&mut tmp, x.as_ptr());
        blst_fr_from_scalar(&mut out, &tmp);
    }
    FsFr(out)
}

#[allow(clippy::useless_conversion)]
pub fn bytes_to_blob(bytes: &[u8]) -> Result<Vec<FsFr>, String> {
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
                .and_then(FsFr::from_bytes)
        })
        .collect()
}

fn is_trusted_setup_in_lagrange_form(g1_values: &Vec<FsG1>, g2_values: &Vec<FsG2>) -> bool {
    if g1_values.len() < 2 || g2_values.len() < 2 {
        return false;
    }

    let is_monotomial_form =
        pairings_verify(&g1_values[1], &g2_values[0], &g1_values[0], &g2_values[1]);

    !is_monotomial_form
}

#[allow(clippy::useless_conversion)]
fn load_trusted_setup_rust(g1_bytes: &[u8], g2_bytes: &[u8]) -> Result<FsKZGSettings, String> {
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
        .map(FsG1::from_bytes)
        .collect::<Result<Vec<FsG1>, String>>()?;

    let g2_values = g2_bytes
        .chunks(BYTES_PER_G2)
        .map(FsG2::from_bytes)
        .collect::<Result<Vec<FsG2>, String>>()?;

    // Sanity check, that user is not trying to load old trusted setup file
    if !is_trusted_setup_in_lagrange_form(&g1_values, &g2_values) {
        return Err(String::from("Trusted setup is not in Lagrange form"));
    }

    let mut max_scale: usize = 0;
    while (1 << max_scale) < num_g1_points {
        max_scale += 1;
    }

    let fs = FsFFTSettings::new(max_scale)?;

    reverse_bit_order(&mut g1_values)?;

    Ok(FsKZGSettings {
        secret_g1: g1_values,
        secret_g2: g2_values,
        fs,
    })
}

#[cfg(feature = "std")]
pub fn load_trusted_setup_filename_rust(filepath: &str) -> Result<FsKZGSettings, String> {
    let mut file = File::open(filepath).map_err(|_| "Unable to open file".to_string())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|_| "Unable to read file".to_string())?;

    let (g1_bytes, g2_bytes) = load_trusted_setup_string(&contents)?;
    load_trusted_setup_rust(g1_bytes.as_slice(), g2_bytes.as_slice())
}

fn fr_batch_inv(out: &mut [FsFr], a: &[FsFr], len: usize) -> Result<(), String> {
    if len == 0 {
        return Err(String::from("Length is less than 0."));
    }

    if a == out {
        return Err(String::from("Destination is the same as source."));
    }

    let mut accumulator = FsFr::one();

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

fn g1_lincomb(points: &[FsG1], scalars: &[FsFr], length: usize) -> FsG1 {
    let mut out = FsG1::default();
    g1_linear_combination(&mut out, points, scalars, length);
    out
}

pub fn blob_to_kzg_commitment_rust(
    blob: &[FsFr],
    settings: &FsKZGSettings,
) -> Result<FsG1, String> {
    let polynomial = blob_to_polynomial_rust(blob)?;

    Ok(poly_to_kzg_commitment(&polynomial, settings))
}

pub fn verify_kzg_proof_rust(
    commitment: &FsG1,
    z: &FsFr,
    y: &FsFr,
    proof: &FsG1,
    s: &FsKZGSettings,
) -> Result<bool, String> {
    if !commitment.is_inf() && !commitment.is_valid() {
        return Err("Invalid commitment".to_string());
    }
    if !proof.is_inf() && !proof.is_valid() {
        return Err("Invalid proof".to_string());
    }

    s.check_proof_single(commitment, proof, z, y)
}

pub fn verify_kzg_proof_batch(
    commitments_g1: &[FsG1],
    zs_fr: &[FsFr],
    ys_fr: &[FsFr],
    proofs_g1: &[FsG1],
    ts: &FsKZGSettings,
) -> Result<bool, String> {
    let n = commitments_g1.len();
    let mut c_minus_y: Vec<FsG1> = Vec::with_capacity(n);
    let mut r_times_z: Vec<FsFr> = Vec::with_capacity(n);

    // Compute the random lincomb challenges
    let r_powers = compute_r_powers(commitments_g1, zs_fr, ys_fr, proofs_g1)?;

    // Compute \sum r^i * Proof_i
    let proof_lincomb = g1_lincomb(proofs_g1, &r_powers, n);

    for i in 0..n {
        // Get [y_i]
        let ys_encrypted = FsG1::generator().mul(&ys_fr[i]);
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
    Ok(pairings_verify(
        &proof_lincomb,
        &ts.secret_g2[1],
        &rhs_g1,
        &FsG2::generator(),
    ))
}

pub fn compute_kzg_proof_rust(
    blob: &[FsFr],
    z: &FsFr,
    s: &FsKZGSettings,
) -> Result<(FsG1, FsFr), String> {
    let polynomial = blob_to_polynomial_rust(blob)?;
    let y = evaluate_polynomial_in_evaluation_form_rust(&polynomial, z, s)?;

    let mut tmp: FsFr;
    let roots_of_unity: &Vec<FsFr> = &s.fs.roots_of_unity;

    let mut m: usize = 0;
    let mut q: FsPoly = FsPoly::new(FIELD_ELEMENTS_PER_BLOB);

    let mut inverses_in: Vec<FsFr> = vec![FsFr::default(); FIELD_ELEMENTS_PER_BLOB];
    let mut inverses: Vec<FsFr> = vec![FsFr::default(); FIELD_ELEMENTS_PER_BLOB];

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        if z.equals(&roots_of_unity[i]) {
            // We are asked to compute a KZG proof inside the domain
            m = i + 1;
            inverses_in[i] = FsFr::one();
            continue;
        }
        // (p_i - y) / (ω_i - z)
        q.coeffs[i] = polynomial.coeffs[i].sub(&y);
        inverses_in[i] = roots_of_unity[i].sub(z);
    }

    fr_batch_inv(&mut inverses, &inverses_in, FIELD_ELEMENTS_PER_BLOB)?;

    for (i, inverse) in inverses.iter().enumerate().take(FIELD_ELEMENTS_PER_BLOB) {
        q.coeffs[i] = q.coeffs[i].mul(inverse);
    }

    if m != 0 {
        // ω_{m-1} == z
        m -= 1;
        q.coeffs[m] = FsFr::zero();
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
            tmp = polynomial.coeffs[i].sub(&y);
            tmp = tmp.mul(&roots_of_unity[i]);
            // Do the division: (p_i - y) * ω_i / (z * (z - ω_i))
            tmp = tmp.mul(&inverses[i]);
            q.coeffs[m] = q.coeffs[m].add(&tmp);
        }
    }

    let proof = g1_lincomb(&s.secret_g1, &q.coeffs, FIELD_ELEMENTS_PER_BLOB);
    Ok((proof, y))
}

pub fn evaluate_polynomial_in_evaluation_form_rust(
    p: &FsPoly,
    x: &FsFr,
    s: &FsKZGSettings,
) -> Result<FsFr, String> {
    if p.coeffs.len() != FIELD_ELEMENTS_PER_BLOB {
        return Err(String::from("Incorrect field elements count."));
    }

    let roots_of_unity: &Vec<FsFr> = &s.fs.roots_of_unity;
    let mut inverses_in: Vec<FsFr> = vec![FsFr::default(); FIELD_ELEMENTS_PER_BLOB];
    let mut inverses: Vec<FsFr> = vec![FsFr::default(); FIELD_ELEMENTS_PER_BLOB];

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        if x.equals(&roots_of_unity[i]) {
            return Ok(p.get_coeff_at(i));
        }
        inverses_in[i] = x.sub(&roots_of_unity[i]);
    }

    fr_batch_inv(&mut inverses, &inverses_in, FIELD_ELEMENTS_PER_BLOB)?;

    let mut tmp: FsFr;
    let mut out = FsFr::zero();

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        tmp = inverses[i].mul(&roots_of_unity[i]);
        tmp = tmp.mul(&p.coeffs[i]);
        out = out.add(&tmp);
    }

    tmp = FsFr::from_u64(FIELD_ELEMENTS_PER_BLOB as u64);
    out = match out.div(&tmp) {
        Ok(value) => value,
        Err(err) => return Err(err),
    };
    tmp = x.pow(FIELD_ELEMENTS_PER_BLOB);
    tmp = tmp.sub(&FsFr::one());
    out = out.mul(&tmp);
    Ok(out)
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

fn compute_challenge(blob: &[FsFr], commitment: &FsG1) -> FsFr {
    let mut bytes: Vec<u8> = vec![0; CHALLENGE_INPUT_SIZE];

    // Copy domain separator
    bytes[..16].copy_from_slice(&FIAT_SHAMIR_PROTOCOL_DOMAIN);
    // Set all other bytes of this 16-byte (big-endian) field to zero
    bytes_of_uint64(&mut bytes[16..24], 0);
    bytes_of_uint64(&mut bytes[24..32], FIELD_ELEMENTS_PER_BLOB as u64);

    // Copy blob
    for (i, field) in blob.iter().enumerate() {
        let v = field.to_bytes();
        let size = (32 + i * BYTES_PER_FIELD_ELEMENT)..(32 + (i + 1) * BYTES_PER_FIELD_ELEMENT);

        assert!(size.len() == v.len());

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

fn compute_r_powers(
    commitments_g1: &[FsG1],
    zs_fr: &[FsFr],
    ys_fr: &[FsFr],
    proofs_g1: &[FsG1],
) -> Result<Vec<FsFr>, String> {
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

pub fn blob_to_polynomial_rust(blob: &[FsFr]) -> Result<FsPoly, String> {
    if blob.len() != FIELD_ELEMENTS_PER_BLOB {
        return Err(String::from("Blob length must be FIELD_ELEMENTS_PER_BLOB"));
    }
    let mut p: FsPoly = FsPoly::new(FIELD_ELEMENTS_PER_BLOB);
    p.coeffs = blob.to_vec();
    Ok(p)
}

fn poly_to_kzg_commitment(p: &FsPoly, s: &FsKZGSettings) -> FsG1 {
    g1_lincomb(&s.secret_g1, &p.coeffs, FIELD_ELEMENTS_PER_BLOB)
}

pub fn compute_blob_kzg_proof_rust(
    blob: &[FsFr],
    commitment: &FsG1,
    ts: &FsKZGSettings,
) -> Result<FsG1, String> {
    if !commitment.is_inf() && !commitment.is_valid() {
        return Err("Invalid commitment".to_string());
    }

    let evaluation_challenge_fr = compute_challenge(blob, commitment);
    let (proof, _) = compute_kzg_proof_rust(blob, &evaluation_challenge_fr, ts)?;
    Ok(proof)
}

pub fn verify_blob_kzg_proof_rust(
    blob: &[FsFr],
    commitment_g1: &FsG1,
    proof_g1: &FsG1,
    ts: &FsKZGSettings,
) -> Result<bool, String> {
    if !commitment_g1.is_inf() && !commitment_g1.is_valid() {
        return Err("Invalid commitment".to_string());
    }
    if !proof_g1.is_inf() && !proof_g1.is_valid() {
        return Err("Invalid proof".to_string());
    }

    let polynomial = blob_to_polynomial_rust(blob)?;
    let evaluation_challenge_fr = compute_challenge(blob, commitment_g1);
    let y_fr =
        evaluate_polynomial_in_evaluation_form_rust(&polynomial, &evaluation_challenge_fr, ts)?;
    verify_kzg_proof_rust(commitment_g1, &evaluation_challenge_fr, &y_fr, proof_g1, ts)
}

fn compute_challenges_and_evaluate_polynomial(
    blobs: &[Vec<FsFr>],
    commitments_g1: &[FsG1],
    ts: &FsKZGSettings,
) -> Result<(Vec<FsFr>, Vec<FsFr>), String> {
    let mut evaluation_challenges_fr = Vec::with_capacity(blobs.len());
    let mut ys_fr = Vec::with_capacity(blobs.len());

    for i in 0..blobs.len() {
        let polynomial = blob_to_polynomial_rust(&blobs[i])?;
        let evaluation_challenge_fr = compute_challenge(&blobs[i], &commitments_g1[i]);
        let y_fr =
            evaluate_polynomial_in_evaluation_form_rust(&polynomial, &evaluation_challenge_fr, ts)?;

        evaluation_challenges_fr.push(evaluation_challenge_fr);
        ys_fr.push(y_fr);
    }

    Ok((evaluation_challenges_fr, ys_fr))
}

fn validate_batched_input(commitments: &[FsG1], proofs: &[FsG1]) -> Result<(), String> {
    let invalid_commitment = cfg_into_iter!(commitments)
        .any(|&commitment| !commitment.is_inf() && !commitment.is_valid());
    let invalid_proof = cfg_into_iter!(proofs).any(|&proof| !proof.is_inf() && !proof.is_valid());

    if invalid_commitment {
        return Err("Invalid commitment".to_string());
    }
    if invalid_proof {
        return Err("Invalid proof".to_string());
    }

    Ok(())
}

pub fn verify_blob_kzg_proof_batch_rust(
    blobs: &[Vec<FsFr>],
    commitments_g1: &[FsG1],
    proofs_g1: &[FsG1],
    ts: &FsKZGSettings,
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

fn fft_settings_to_rust(c_settings: *const CKZGSettings) -> Result<FsFFTSettings, String> {
    let settings = unsafe { &*c_settings };

    let roots_of_unity = unsafe {
        core::slice::from_raw_parts(settings.roots_of_unity, settings.max_width as usize)
            .iter()
            .map(|r| FsFr(*r))
            .collect::<Vec<FsFr>>()
    };
    let mut expanded_roots_of_unity = roots_of_unity.clone();
    reverse_bit_order(&mut expanded_roots_of_unity)?;
    expanded_roots_of_unity.push(FsFr::one());
    let mut reverse_roots_of_unity = expanded_roots_of_unity.clone();
    reverse_roots_of_unity.reverse();

    let mut first_root = expanded_roots_of_unity[1];
    let first_root_arr = [first_root; 1];
    first_root = first_root_arr[0];

    Ok(FsFFTSettings {
        max_width: settings.max_width as usize,
        root_of_unity: first_root,
        expanded_roots_of_unity,
        reverse_roots_of_unity,
        roots_of_unity,
    })
}

fn kzg_settings_to_rust(c_settings: &CKZGSettings) -> Result<FsKZGSettings, String> {
    let secret_g1 = unsafe {
        core::slice::from_raw_parts(c_settings.g1_values, TRUSTED_SETUP_NUM_G1_POINTS)
            .iter()
            .map(|r| FsG1(*r))
            .collect::<Vec<FsG1>>()
    };
    Ok(FsKZGSettings {
        fs: fft_settings_to_rust(c_settings)?,
        secret_g1,
        secret_g2: unsafe {
            core::slice::from_raw_parts(c_settings.g2_values, TRUSTED_SETUP_NUM_G2_POINTS)
                .iter()
                .map(|r| FsG2(*r))
                .collect::<Vec<FsG2>>()
        },
    })
}

fn kzg_settings_to_c(rust_settings: &FsKZGSettings) -> CKZGSettings {
    let g1_val = rust_settings
        .secret_g1
        .iter()
        .map(|r| r.0)
        .collect::<Vec<blst_p1>>();
    let g1_val = Box::new(g1_val);
    let g2_val = rust_settings
        .secret_g2
        .iter()
        .map(|r| r.0)
        .collect::<Vec<blst_p2>>();
    let x = g2_val.into_boxed_slice();
    let stat_ref = Box::leak(x);
    let v = Box::into_raw(g1_val);

    let roots_of_unity = Box::new(
        rust_settings
            .fs
            .roots_of_unity
            .iter()
            .map(|r| r.0)
            .collect::<Vec<blst_fr>>(),
    );

    CKZGSettings {
        max_width: rust_settings.fs.max_width as u64,
        roots_of_unity: unsafe { (*Box::into_raw(roots_of_unity)).as_mut_ptr() },
        g1_values: unsafe { (*v).as_mut_ptr() },
        g2_values: stat_ref.as_mut_ptr(),
    }
}

unsafe fn deserialize_blob(blob: *const Blob) -> Result<Vec<FsFr>, C_KZG_RET> {
    (*blob)
        .bytes
        .chunks(BYTES_PER_FIELD_ELEMENT)
        .map(|chunk| {
            let mut bytes = [0u8; BYTES_PER_FIELD_ELEMENT];
            bytes.copy_from_slice(chunk);
            if let Ok(result) = FsFr::from_bytes(&bytes) {
                Ok(result)
            } else {
                Err(C_KZG_RET_BADARGS)
            }
        })
        .collect::<Result<Vec<FsFr>, C_KZG_RET>>()
}

macro_rules! handle_ckzg_badargs {
    ($x: expr) => {
        match $x {
            Ok(value) => value,
            Err(_) => return C_KZG_RET_BADARGS,
        }
    };
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn blob_to_kzg_commitment(
    out: *mut KZGCommitment,
    blob: *const Blob,
    s: &CKZGSettings,
) -> C_KZG_RET {
    if TRUSTED_SETUP_NUM_G1_POINTS == 0 {
        // FIXME: load_trusted_setup should set this value, but if not, it fails
        TRUSTED_SETUP_NUM_G1_POINTS = FIELD_ELEMENTS_PER_BLOB
    };

    let deserialized_blob = handle_ckzg_badargs!(deserialize_blob(blob));
    let settings = handle_ckzg_badargs!(kzg_settings_to_rust(s));
    let tmp = handle_ckzg_badargs!(blob_to_kzg_commitment_rust(&deserialized_blob, &settings));

    (*out).bytes = tmp.to_bytes();
    C_KZG_RET_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn load_trusted_setup(
    out: *mut CKZGSettings,
    g1_bytes: *const u8,
    n1: usize,
    g2_bytes: *const u8,
    n2: usize,
) -> C_KZG_RET {
    let g1_bytes = core::slice::from_raw_parts(g1_bytes, n1 * BYTES_PER_G1);
    let g2_bytes = core::slice::from_raw_parts(g2_bytes, n2 * BYTES_PER_G2);
    TRUSTED_SETUP_NUM_G1_POINTS = g1_bytes.len() / BYTES_PER_G1;
    let settings = handle_ckzg_badargs!(load_trusted_setup_rust(g1_bytes, g2_bytes));

    *out = kzg_settings_to_c(&settings);
    C_KZG_RET_OK
}

/// # Safety
#[cfg(feature = "std")]
#[no_mangle]
pub unsafe extern "C" fn load_trusted_setup_file(
    out: *mut CKZGSettings,
    in_: *mut FILE,
) -> C_KZG_RET {
    let mut buf = vec![0u8; 1024 * 1024];
    let len: usize = libc::fread(buf.as_mut_ptr() as *mut libc::c_void, 1, buf.len(), in_);
    let s = handle_ckzg_badargs!(String::from_utf8(buf[..len].to_vec()));
    let (g1_bytes, g2_bytes) = handle_ckzg_badargs!(load_trusted_setup_string(&s));
    TRUSTED_SETUP_NUM_G1_POINTS = g1_bytes.len() / BYTES_PER_G1;
    if TRUSTED_SETUP_NUM_G1_POINTS != FIELD_ELEMENTS_PER_BLOB {
        // Helps pass the Java test "shouldThrowExceptionOnIncorrectTrustedSetupFromFile",
        // as well as 5 others that pass only if this one passes (likely because Java doesn't
        // deallocate its KZGSettings pointer when no exception is thrown).
        return C_KZG_RET_BADARGS;
    }
    let settings = handle_ckzg_badargs!(load_trusted_setup_rust(
        g1_bytes.as_slice(),
        g2_bytes.as_slice()
    ));

    *out = kzg_settings_to_c(&settings);
    C_KZG_RET_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn compute_blob_kzg_proof(
    out: *mut KZGProof,
    blob: *const Blob,
    commitment_bytes: *const Bytes48,
    s: &CKZGSettings,
) -> C_KZG_RET {
    let deserialized_blob = match deserialize_blob(blob) {
        Ok(value) => value,
        Err(err) => return err,
    };

    let commitment_g1 = handle_ckzg_badargs!(FsG1::from_bytes(&(*commitment_bytes).bytes));
    let settings = handle_ckzg_badargs!(kzg_settings_to_rust(s));
    let proof = handle_ckzg_badargs!(compute_blob_kzg_proof_rust(
        &deserialized_blob,
        &commitment_g1,
        &settings
    ));

    (*out).bytes = proof.to_bytes();
    C_KZG_RET_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn free_trusted_setup(s: *mut CKZGSettings) {
    if s.is_null() {
        return;
    }

    let max_width = (*s).max_width as usize;
    let roots = Box::from_raw(core::slice::from_raw_parts_mut(
        (*s).roots_of_unity,
        max_width,
    ));
    drop(roots);
    (*s).roots_of_unity = null_mut();

    let g1 = Box::from_raw(core::slice::from_raw_parts_mut(
        (*s).g1_values,
        TRUSTED_SETUP_NUM_G1_POINTS,
    ));
    drop(g1);
    (*s).g1_values = null_mut();

    let g2 = Box::from_raw(core::slice::from_raw_parts_mut(
        (*s).g2_values,
        TRUSTED_SETUP_NUM_G2_POINTS,
    ));
    drop(g2);
    (*s).g2_values = null_mut();
    (*s).max_width = 0;
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn verify_kzg_proof(
    ok: *mut bool,
    commitment_bytes: *const Bytes48,
    z_bytes: *const Bytes32,
    y_bytes: *const Bytes32,
    proof_bytes: *const Bytes48,
    s: &CKZGSettings,
) -> C_KZG_RET {
    let frz = handle_ckzg_badargs!(FsFr::from_bytes(&(*z_bytes).bytes));
    let fry = handle_ckzg_badargs!(FsFr::from_bytes(&(*y_bytes).bytes));
    let g1commitment = handle_ckzg_badargs!(FsG1::from_bytes(&(*commitment_bytes).bytes));
    let g1proof = handle_ckzg_badargs!(FsG1::from_bytes(&(*proof_bytes).bytes));

    let settings = handle_ckzg_badargs!(kzg_settings_to_rust(s));

    let result = handle_ckzg_badargs!(verify_kzg_proof_rust(
        &g1commitment,
        &frz,
        &fry,
        &g1proof,
        &settings
    ));

    *ok = result;
    C_KZG_RET_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn verify_blob_kzg_proof(
    ok: *mut bool,
    blob: *const Blob,
    commitment_bytes: *const Bytes48,
    proof_bytes: *const Bytes48,
    s: &CKZGSettings,
) -> C_KZG_RET {
    let deserialized_blob = handle_ckzg_badargs!(deserialize_blob(blob));
    let commitment_g1 = handle_ckzg_badargs!(FsG1::from_bytes(&(*commitment_bytes).bytes));
    let proof_g1 = handle_ckzg_badargs!(FsG1::from_bytes(&(*proof_bytes).bytes));
    let settings = handle_ckzg_badargs!(kzg_settings_to_rust(s));

    let result = handle_ckzg_badargs!(verify_blob_kzg_proof_rust(
        &deserialized_blob,
        &commitment_g1,
        &proof_g1,
        &settings,
    ));

    *ok = result;
    C_KZG_RET_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn verify_blob_kzg_proof_batch(
    ok: *mut bool,
    blobs: *const Blob,
    commitments_bytes: *const Bytes48,
    proofs_bytes: *const Bytes48,
    n: usize,
    s: &CKZGSettings,
) -> C_KZG_RET {
    let raw_blobs = core::slice::from_raw_parts(blobs, n);
    let raw_commitments = core::slice::from_raw_parts(commitments_bytes, n);
    let raw_proofs = core::slice::from_raw_parts(proofs_bytes, n);

    let deserialized_blobs: Result<Vec<Vec<FsFr>>, C_KZG_RET> = cfg_into_iter!(raw_blobs)
        .map(|raw_blob| deserialize_blob(raw_blob).map_err(|_| C_KZG_RET_BADARGS))
        .collect();

    let commitments_g1: Result<Vec<FsG1>, C_KZG_RET> = cfg_into_iter!(raw_commitments)
        .map(|raw_commitment| {
            FsG1::from_bytes(&raw_commitment.bytes).map_err(|_| C_KZG_RET_BADARGS)
        })
        .collect();

    let proofs_g1: Result<Vec<FsG1>, C_KZG_RET> = cfg_into_iter!(raw_proofs)
        .map(|raw_proof| FsG1::from_bytes(&raw_proof.bytes).map_err(|_| C_KZG_RET_BADARGS))
        .collect();

    if let (Ok(blobs), Ok(commitments), Ok(proofs)) =
        (deserialized_blobs, commitments_g1, proofs_g1)
    {
        let settings = match kzg_settings_to_rust(s) {
            Ok(value) => value,
            Err(_) => return C_KZG_RET_BADARGS,
        };

        let result =
            verify_blob_kzg_proof_batch_rust(blobs.as_slice(), &commitments, &proofs, &settings);

        if let Ok(result) = result {
            *ok = result;
            C_KZG_RET_OK
        } else {
            C_KZG_RET_BADARGS
        }
    } else {
        *ok = false;
        C_KZG_RET_BADARGS
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn compute_kzg_proof(
    proof_out: *mut KZGProof,
    y_out: *mut Bytes32,
    blob: *const Blob,
    z_bytes: *const Bytes32,
    s: &CKZGSettings,
) -> C_KZG_RET {
    let deserialized_blob = match deserialize_blob(blob) {
        Ok(value) => value,
        Err(err) => return err,
    };

    let frz = match FsFr::from_bytes(&(*z_bytes).bytes) {
        Ok(value) => value,
        Err(_) => return C_KZG_RET_BADARGS,
    };

    let settings = match kzg_settings_to_rust(s) {
        Ok(value) => value,
        Err(_) => return C_KZG_RET_BADARGS,
    };

    let (proof_out_tmp, fry_tmp) = match compute_kzg_proof_rust(&deserialized_blob, &frz, &settings)
    {
        Ok(value) => value,
        Err(_) => return C_KZG_RET_BADARGS,
    };

    (*proof_out).bytes = proof_out_tmp.to_bytes();
    (*y_out).bytes = fry_tmp.to_bytes();
    C_KZG_RET_OK
}

#[cfg(test)]
mod tests {
    use kzg_bench::tests::utils::get_trusted_setup_path;

    use crate::eip_4844::{kzg_settings_to_c, kzg_settings_to_rust};

    use super::load_trusted_setup_filename_rust;

    #[test]
    fn kzg_settings_to_rust_check_conversion() {
        let settings = load_trusted_setup_filename_rust(get_trusted_setup_path().as_str());

        assert!(settings.is_ok());

        let settings = settings.unwrap();

        let converted_settings = kzg_settings_to_rust(&kzg_settings_to_c(&settings)).unwrap();

        assert_eq!(
            settings.fs.root_of_unity,
            converted_settings.fs.root_of_unity
        );
        assert_eq!(
            settings.fs.roots_of_unity,
            converted_settings.fs.roots_of_unity
        );
        assert_eq!(
            settings.fs.expanded_roots_of_unity,
            converted_settings.fs.expanded_roots_of_unity
        );
        assert_eq!(
            settings.fs.reverse_roots_of_unity,
            converted_settings.fs.reverse_roots_of_unity
        );
    }
}
