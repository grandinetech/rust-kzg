use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use std::ops::Sub;
use blst::{blst_fr, blst_p1, blst_p2};

use crate::fk20::reverse_bit_order;
use crate::kzg_proofs::{check_proof_single, KZGSettings};
use crate::kzg_types::{pairings_verify, ZkG1Projective, ZkG2Projective};
use crate::poly::KzgPoly;
use crate::zkfr::blsScalar;
use kzg::eip_4844::{
    bytes_of_uint64, Blob, hash, load_trusted_setup_string, KZGCommitment,
    BYTES_PER_BLOB, BYTES_PER_COMMITMENT, BYTES_PER_FIELD_ELEMENT, BYTES_PER_G1,
    BYTES_PER_G2, BYTES_PER_PROOF, CHALLENGE_INPUT_SIZE, FIAT_SHAMIR_PROTOCOL_DOMAIN,
    FIELD_ELEMENTS_PER_BLOB, RANDOM_CHALLENGE_KZG_BATCH_DOMAIN,
    TRUSTED_SETUP_NUM_G1_POINTS, TRUSTED_SETUP_NUM_G2_POINTS,
    Bytes32, Bytes48, CFFTSettings, CKZGSettings, KZGProof,
    C_KZG_RET, C_KZG_RET_BADARGS, C_KZG_RET_OK,
};

use kzg::{cfg_into_iter, FFTSettings, Fr, Poly, FFTG1, G1, G2};

use crate::curve::multiscalar_mul::msm_variable_base;
use crate::fftsettings::ZkFFTSettings;

#[cfg(feature = "parallel")]
use rayon::prelude::*;
use crate::curve::g1::G1Projective;
use crate::curve::g2::G2Projective;

pub fn hash_to_bls_field(x: &[u8; BYTES_PER_FIELD_ELEMENT]) -> blsScalar {
    Fr::from_bytes(x).unwrap()
}

#[allow(clippy::useless_conversion)]
pub fn bytes_to_blob(bytes: &[u8]) -> Result<Vec<blsScalar>, String> {
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
fn load_trusted_setup_rust(g1_bytes: &[u8], g2_bytes: &[u8]) -> KZGSettings {
    let num_g1_points = g1_bytes.len() / BYTES_PER_G1;

    assert_eq!(g1_bytes.len() / BYTES_PER_G1, FIELD_ELEMENTS_PER_BLOB);
    assert_eq!(g2_bytes.len() / BYTES_PER_G2, TRUSTED_SETUP_NUM_G2_POINTS);

    let g1_projectives: Vec<ZkG1Projective> = g1_bytes
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

    let g2_values: Vec<ZkG2Projective> = g2_bytes
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

    let fs = ZkFFTSettings::new(max_scale).unwrap();
    let mut g1_values = fs.fft_g1(&g1_projectives, true).unwrap();
    reverse_bit_order(&mut g1_values);

    KZGSettings {
        secret_g1: g1_values,
        secret_g2: g2_values,
        fs,
        length: num_g1_points as u64,
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

fn fr_batch_inv(out: &mut [blsScalar], a: &[blsScalar], len: usize) {
    assert!(len > 0);

    let mut accumulator = blsScalar::one();

    for i in 0..len {
        out[i] = accumulator;
        accumulator = accumulator.mul(&a[i]);
    }

    accumulator = accumulator.eucl_inverse();

    for i in (0..len).rev() {
        out[i] = out[i].mul(&accumulator);
        accumulator = accumulator.mul(&a[i]);
    }
}

fn g1_lincomb(points: &[ZkG1Projective], scalars: &[blsScalar], _length: usize) -> ZkG1Projective {
    msm_variable_base(points, scalars)
}

pub fn compute_powers(base: &blsScalar, num_powers: usize) -> Vec<blsScalar> {
    let mut powers: Vec<blsScalar> = vec![blsScalar::default(); num_powers];
    powers[0] = blsScalar::one();
    for i in 1..num_powers {
        powers[i] = powers[i - 1].mul(base);
    }
    powers
}

pub fn blob_to_kzg_commitment(blob: &[blsScalar], s: &KZGSettings) -> ZkG1Projective {
    let p = blob_to_polynomial(blob);
    poly_to_kzg_commitment(&p, s)
}

pub fn verify_kzg_proof(
    commitment: &ZkG1Projective,
    z: &blsScalar,
    y: &blsScalar,
    proof: &ZkG1Projective,
    s: &KZGSettings,
) -> Result<bool, String> {
    if !commitment.is_valid() {
        return Err("Invalid commitment".to_string());
    }
    if !proof.is_valid() {
        return Err("Invalid proof".to_string());
    }

    Ok(check_proof_single(commitment, proof, z, y, s).unwrap_or(false))
}

pub fn verify_kzg_proof_batch(
    commitments_g1: &[ZkG1Projective],
    zs_fr: &[blsScalar],
    ys_fr: &[blsScalar],
    proofs_g1: &[ZkG1Projective],
    ts: &KZGSettings,
) -> bool {
    let n = commitments_g1.len();
    let mut c_minus_y: Vec<ZkG1Projective> = Vec::new();
    let mut r_times_z: Vec<blsScalar> = Vec::new();

    // Compute the random lincomb challenges
    let r_powers = compute_r_powers(commitments_g1, zs_fr, ys_fr, proofs_g1);

    // Compute \sum r^i * Proof_i
    let proof_lincomb = g1_lincomb(proofs_g1, &r_powers, n);

    for i in 0..n {
        // Get [y_i]
        let ys_encrypted = ZkG1Projective::generator().mul(&ys_fr[i]);
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
        &ts.secret_g2[1],
        &rhs_g1,
        &ZkG2Projective::generator(),
    )
}

pub fn compute_kzg_proof(
    blob: &[blsScalar],
    z: &blsScalar,
    s: &KZGSettings,
) -> (ZkG1Projective, blsScalar) {
    assert_eq!(blob.len(), FIELD_ELEMENTS_PER_BLOB);

    let polynomial = blob_to_polynomial(blob);
    let y = evaluate_polynomial_in_evaluation_form(&polynomial, z, s);

    let mut tmp: blsScalar;
    let roots_of_unity: &Vec<blsScalar> = &s.fs.roots_of_unity;

    let mut m: usize = 0;
    let mut q: KzgPoly = KzgPoly::new(FIELD_ELEMENTS_PER_BLOB).unwrap();

    let mut inverses_in: Vec<blsScalar> = vec![blsScalar::default(); FIELD_ELEMENTS_PER_BLOB];
    let mut inverses: Vec<blsScalar> = vec![blsScalar::default(); FIELD_ELEMENTS_PER_BLOB];

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        if z.equals(&roots_of_unity[i]) {
            // We are asked to compute a KZG proof inside the domain
            m = i + 1;
            inverses_in[i] = blsScalar::one();
            continue;
        }
        // (p_i - y) / (ω_i - z)
        q.coeffs[i] = polynomial.coeffs[i].sub(&y);
        inverses_in[i] = roots_of_unity[i].sub(z);
    }

    fr_batch_inv(&mut inverses, &inverses_in, FIELD_ELEMENTS_PER_BLOB);

    for (i, inverse) in inverses.iter().enumerate().take(FIELD_ELEMENTS_PER_BLOB) {
        q.coeffs[i] = q.coeffs[i].mul(inverse);
    }

    if m != 0 {
        // ω_{m-1} == z
        m -= 1;
        q.coeffs[m] = blsScalar::zero();
        for i in 0..FIELD_ELEMENTS_PER_BLOB {
            if i == m {
                continue;
            }
            // Build denominator: z * (z - ω_i)
            tmp = z.sub(&roots_of_unity[i]);
            inverses_in[i] = tmp.mul(z);
        }

        fr_batch_inv(&mut inverses, &inverses_in, FIELD_ELEMENTS_PER_BLOB);

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
    (proof, y)
}

pub fn evaluate_polynomial_in_evaluation_form(
    p: &KzgPoly,
    x: &blsScalar,
    s: &KZGSettings,
) -> blsScalar {
    assert_eq!(p.coeffs.len(), FIELD_ELEMENTS_PER_BLOB);

    let roots_of_unity: &Vec<blsScalar> = &s.fs.roots_of_unity;
    let mut inverses_in: Vec<blsScalar> = vec![blsScalar::default(); FIELD_ELEMENTS_PER_BLOB];
    let mut inverses: Vec<blsScalar> = vec![blsScalar::default(); FIELD_ELEMENTS_PER_BLOB];

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        if x.equals(&roots_of_unity[i]) {
            return p.get_coeff_at(i);
        }
        inverses_in[i] = x.sub(&roots_of_unity[i]);
    }

    fr_batch_inv(&mut inverses, &inverses_in, FIELD_ELEMENTS_PER_BLOB);

    let mut tmp: blsScalar;
    let mut out = blsScalar::zero();

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        tmp = inverses[i].mul(&roots_of_unity[i]);
        tmp = tmp.mul(&p.coeffs[i]);
        out = out.add(&tmp);
    }

    let arr: [u64; 4] = [FIELD_ELEMENTS_PER_BLOB as u64, 0, 0, 0];
    tmp = blsScalar::from_u64_arr(&arr);
    out = out.div(&tmp).unwrap();
    tmp = x.pow(&arr);
    tmp = tmp.sub(&blsScalar::one());
    out = out.mul(&tmp);
    out
}

fn compute_challenge(blob: &[blsScalar], commitment: &ZkG1Projective) -> blsScalar {
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
    let v = G1::to_bytes(commitment);
    for i in 0..v.len() {
        bytes[32 + BYTES_PER_BLOB + i] = v[i];
    }

    // Now let's create the challenge!
    let eval_challenge = hash(&bytes);
    hash_to_bls_field(&eval_challenge)
}

fn compute_r_powers(
    commitments_g1: &[ZkG1Projective],
    zs_fr: &[blsScalar],
    ys_fr: &[blsScalar],
    proofs_g1: &[ZkG1Projective],
) -> Vec<blsScalar> {
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
        let v = G1::to_bytes(&commitments_g1[i]);
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
        let v = G1::to_bytes(&proofs_g1[i]);
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

pub fn blob_to_polynomial(blob: &[blsScalar]) -> KzgPoly {
    assert_eq!(blob.len(), FIELD_ELEMENTS_PER_BLOB);
    let mut p: KzgPoly = KzgPoly::new(FIELD_ELEMENTS_PER_BLOB).unwrap();
    p.coeffs = blob.to_vec();
    p
}

fn poly_to_kzg_commitment(p: &KzgPoly, s: &KZGSettings) -> ZkG1Projective {
    assert_eq!(p.coeffs.len(), FIELD_ELEMENTS_PER_BLOB);
    g1_lincomb(&s.secret_g1, &p.coeffs, FIELD_ELEMENTS_PER_BLOB)
}

pub fn compute_blob_kzg_proof(
    blob: &[blsScalar],
    commitment: &ZkG1Projective,
    ts: &KZGSettings,
) -> Result<ZkG1Projective, String> {
    if !commitment.is_valid() {
        return Err("Invalid commitment".to_string());
    }

    let evaluation_challenge_fr = compute_challenge(blob, commitment);
    let (proof, _) = compute_kzg_proof(blob, &evaluation_challenge_fr, ts);
    Ok(proof)
}

pub fn verify_blob_kzg_proof(
    blob: &[blsScalar],
    commitment_g1: &ZkG1Projective,
    proof_g1: &ZkG1Projective,
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
    blobs: &[Vec<blsScalar>],
    commitments_g1: &[ZkG1Projective],
    ts: &KZGSettings,
) -> (Vec<blsScalar>, Vec<blsScalar>) {
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

fn validate_batched_input(
    commitments: &[ZkG1Projective],
    proofs: &[ZkG1Projective],
) -> Result<(), String> {
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
    blobs: &[Vec<blsScalar>],
    commitments_g1: &[ZkG1Projective],
    proofs_g1: &[ZkG1Projective],
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
        validate_batched_input(commitments_g1, proofs_g1)?;
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


fn fft_settings_to_rust(c_settings: *const CFFTSettings) -> ZkFFTSettings {
    let settings = unsafe { &*c_settings };
    let mut bl_fr = unsafe { *settings.expanded_roots_of_unity.add(1) };
    let l_values: [u64; 4] = bl_fr.l;
    let mut first_root = blsScalar(l_values);
    let first_root_arr = [first_root; 1];
    first_root = first_root_arr[0];

    let res = ZkFFTSettings {
        max_width: settings.max_width as usize,
        root_of_unity: first_root,
        expanded_roots_of_unity: unsafe {
            core::slice::from_raw_parts(
                settings.expanded_roots_of_unity,
                (settings.max_width + 1) as usize,
            )
                .iter()
                .map(|r| blsScalar(*r.l))
                .collect::<Vec<blsScalar>>()
        },
        reverse_roots_of_unity: unsafe {
            core::slice::from_raw_parts(
                settings.reverse_roots_of_unity,
                (settings.max_width + 1) as usize,
            )
                .iter()
                .map(|r| blsScalar(*r.l))
                .collect::<Vec<blsScalar>>()
        },
        roots_of_unity: unsafe {
            core::slice::from_raw_parts(settings.roots_of_unity, (settings.max_width + 1) as usize)
                .iter()
                .map(|r| blsScalar(*r.l))
                .collect::<Vec<blsScalar>>()
        },
    };

    res
}

unsafe fn deserialize_blob(blob: *const Blob) -> Result<Vec<blsScalar>, C_KZG_RET> {
    (*blob)
        .bytes
        .chunks(BYTES_PER_FIELD_ELEMENT)
        .map(|chunk| {
            let mut bytes = [0u8; BYTES_PER_FIELD_ELEMENT];
            bytes.copy_from_slice(chunk);
            if let Ok(result) = blsScalar::from_bytes(&bytes) {
                Ok(result)
            } else {
                Err(C_KZG_RET_BADARGS)
            }
        })
        .collect::<Result<Vec<blsScalar>, C_KZG_RET>>()
}
fn kzg_settings_to_rust(c_settings: &CKZGSettings) -> KZGSettings {
    let fs = fft_settings_to_rust(c_settings.fs);

    let secret_g1 = unsafe {
        std::slice::from_raw_parts(c_settings.g1_values, TRUSTED_SETUP_NUM_G1_POINTS)
            .iter()
            .map(|r| G1::from(*r))
            .collect::<Vec<G1Projective>>()
    };

    let secret_g2 = unsafe {
        std::slice::from_raw_parts(c_settings.g2_values, TRUSTED_SETUP_NUM_G2_POINTS)
            .iter()
            .map(|r| G2::from(*r))
            .collect::<Vec<G2Projective>>()
    };

    let length = 64;  // zyme PAKLAUSTI DESTITIOJO

    KZGSettings {
        fs,
        secret_g1,
        secret_g2,
        length,
    }
}

fn fft_settings_to_c(rust_settings: &KZGSettings) -> *const CFFTSettings {
    let expanded_roots_of_unity = Box::new(
        rust_settings
            .expanded_roots_of_unity
            .iter()
            .map(|r| r.0)
            .collect::<Vec<blst_fr>>(),
    );
    let reverse_roots_of_unity = Box::new(
        rust_settings
            .reverse_roots_of_unity
            .iter()
            .map(|r| r.0)
            .collect::<Vec<blst_fr>>(),
    );
    let roots_of_unity = Box::new(
        rust_settings
            .roots_of_unity
            .iter()
            .map(|r| r.0)
            .collect::<Vec<blst_fr>>(),
    );

    let b = Box::new(CFFTSettings {
        max_width: rust_settings.max_width as u64,
        expanded_roots_of_unity: unsafe { (*Box::into_raw(expanded_roots_of_unity)).as_mut_ptr() },
        reverse_roots_of_unity: unsafe { (*Box::into_raw(reverse_roots_of_unity)).as_mut_ptr() },
        roots_of_unity: unsafe { (*Box::into_raw(roots_of_unity)).as_mut_ptr() },
    });
    Box::into_raw(b)
}

fn kzg_settings_to_c(rust_settings: &KZGSettings) -> CKZGSettings {
    let g1_val = rust_settings
        .secret_g1
        .iter()
        .map(|r| r)
        .collect::<Vec<blst_p1>>();
    let g1_val = Box::new(g1_val);
    let g2_val = rust_settings
        .secret_g2
        .iter()
        .map(|r| r)
        .collect::<Vec<blst_p2>>();
    let x = g2_val.into_boxed_slice();
    let stat_ref = Box::leak(x);
    let v = Box::into_raw(g1_val);

    CKZGSettings {
        fs: fft_settings_to_c(&rust_settings.fs),
        g1_values: unsafe { (*v).as_mut_ptr() },
        g2_values: stat_ref.as_mut_ptr(),
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn blob_to_kzg_commitment_NEW(
    out: *mut KZGCommitment,
    blob: *const Blob,
    s: &CKZGSettings,
) -> C_KZG_RET {
    let deserialized_blob = deserialize_blob(blob);
    if let Ok(blob_) = deserialized_blob {
        let tmp = blob_to_kzg_commitment(&blob_, &kzg_settings_to_rust(s));
        (*out).bytes = tmp.to_bytes();
        C_KZG_RET_OK
    } else {
        deserialized_blob.err().unwrap()
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn load_trusted_setup_NEW(
    out: *mut CKZGSettings,
    g1_bytes: *const u8,
    n1: usize,
    g2_bytes: *const u8,
    n2: usize,
) -> C_KZG_RET {
    let g1_bytes = core::slice::from_raw_parts(g1_bytes, n1 * BYTES_PER_G1);
    let g2_bytes = core::slice::from_raw_parts(g2_bytes, n2 * BYTES_PER_G2);
    TRUSTED_SETUP_NUM_G1_POINTS = g1_bytes.len() / BYTES_PER_G1;
    let settings = load_trusted_setup_rust(g1_bytes, g2_bytes);
    *out = kzg_settings_to_c(&settings);
    C_KZG_RET_OK
}