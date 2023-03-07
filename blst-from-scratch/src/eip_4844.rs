#![allow(non_camel_case_types)]
use std::convert::TryInto;
use std::ffi::c_char;
use std::fs::File;
use std::io::Read;
use std::ptr::null_mut;

use blst::{
    blst_fr, blst_fr_from_scalar, blst_p1, blst_p1_affine, blst_p1_compress, blst_p1_from_affine,
    blst_p1_uncompress, blst_p2, blst_p2_affine, blst_p2_from_affine, blst_p2_uncompress,
    blst_scalar, blst_scalar_from_lendian, blst_p1_in_g1, BLST_ERROR
};
use kzg::{FFTSettings, Fr, KZGSettings, Poly, FFTG1, G1, G1Mul, G2};

use libc::{c_ulong, fgetc, fgets, strtoul, EOF, FILE};
#[cfg(feature = "parallel")]
use rayon::iter::IntoParallelIterator;
#[cfg(feature = "parallel")]
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use sha2::{Digest, Sha256};

use crate::consts::{
    BYTES_PER_FIELD_ELEMENT, BYTES_PER_PROOF, FIAT_SHAMIR_PROTOCOL_DOMAIN, FIELD_ELEMENTS_PER_BLOB,
    RANDOM_CHALLENGE_KZG_BATCH_DOMAIN
};
use crate::types::fft_settings::FsFFTSettings;
use crate::types::fr::FsFr;
use crate::types::g1::FsG1;

use crate::kzg_proofs::{g1_linear_combination, pairings_verify};
use crate::types::g2::FsG2;
use crate::types::kzg_settings::FsKZGSettings;
use crate::types::poly::FsPoly;
use crate::utils::reverse_bit_order;

pub fn bytes_to_g1_rust(bytes: &[u8; 48usize]) -> Result<FsG1, String> {
    let mut tmp = blst_p1_affine::default();
    let mut g1 = blst_p1::default();
    unsafe {
        // The uncompress routine also checks that the point is on the curve
        if blst_p1_uncompress(&mut tmp, bytes.as_ptr()) != BLST_ERROR::BLST_SUCCESS {
            return Err("blst_p1_uncompress failed".to_string());
        }
        blst_p1_from_affine(&mut g1, &tmp);
        // The point must be on the right subgroup
        if !blst_p1_in_g1(&mut g1) {
            return Err("the point is not in g1 group".to_string());
        }
    }
    Ok(FsG1(g1))
}

pub fn bytes_from_g1_rust(g1: &FsG1) -> [u8; 48usize] {
    let mut out: [u8; 48usize] = [0; 48];
    unsafe {
        blst_p1_compress(out.as_mut_ptr(), &g1.0);
    }
    out
}

pub fn load_trusted_setup_rust(
    g1_bytes: &[u8],
    n1: usize,
    g2_bytes: &[u8],
    _n2: usize,
) -> FsKZGSettings {
    let g1_projectives: Vec<FsG1> = g1_bytes
        .chunks(48)
        .map(|chunk| {
            let mut bytes_array: [u8; 48] = [0; 48];
            bytes_array.copy_from_slice(chunk);
            bytes_to_g1_rust(&bytes_array).unwrap()
        })
        .collect();

    let g2_values: Vec<FsG2> = g2_bytes
        .chunks(96)
        .map(|chunk| {
            let mut bytes_array: [u8; 96] = [0; 96];
            bytes_array.copy_from_slice(chunk);
            let mut tmp = blst_p2_affine::default();
            let mut g2 = blst_p2::default();
            unsafe {
                if blst_p2_uncompress(&mut tmp, bytes_array.as_ptr()) != BLST_ERROR::BLST_SUCCESS {
                    panic!("blst_p2_uncompress failed");
                }
                blst_p2_from_affine(&mut g2, &tmp);
            }
            FsG2(g2)
        })
        .collect();

    let mut max_scale: usize = 0;
    while (1 << max_scale) < n1 {
        max_scale += 1;
    }

    let fs = FsFFTSettings::new(max_scale).unwrap();
    let mut g1_values = fs.fft_g1(&g1_projectives, true).unwrap();

    reverse_bit_order(&mut g1_values);

    FsKZGSettings {
        secret_g1: g1_values,
        secret_g2: g2_values,
        fs,
    }
}

pub fn load_trusted_setup_file_rust(file: &mut File) -> FsKZGSettings {
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    let mut lines = contents.lines();
    let length = lines.next().unwrap().parse::<usize>().unwrap();
    let n2 = lines.next().unwrap().parse::<usize>().unwrap();

    let mut g2_values: Vec<u8> = Vec::new();
    let mut g1_projectives: Vec<u8> = Vec::new();

    for _ in 0..length {
        let line = lines.next().unwrap();
        assert_eq!(line.len(), 96);
        let bytes_array = (0..line.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&line[i..i + 2], 16).unwrap())
            .collect::<Vec<u8>>();
        g1_projectives.extend_from_slice(&bytes_array);
    }

    for _ in 0..n2 {
        let line = lines.next().unwrap();
        assert_eq!(line.len(), 192);
        let bytes = (0..line.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&line[i..i + 2], 16).unwrap())
            .collect::<Vec<u8>>();
        g2_values.extend_from_slice(&bytes);
    }

    load_trusted_setup_rust(&g1_projectives, length, &g2_values, n2)
}

pub fn load_trusted_setup_filename_rust(filename: &str) -> FsKZGSettings {
    let mut file = File::open(filename).expect("Unable to open file");
    load_trusted_setup_file_rust(&mut file)
}

fn fr_batch_inv(out: &mut [FsFr], a: &[FsFr], len: usize) {
    let prod: &mut Vec<FsFr> = &mut vec![FsFr::default(); len];
    let mut i: usize = 1;

    prod[0] = a[0];

    while i < len {
        prod[i] = a[i].mul(&prod[i - 1]);
        i += 1;
    }

    let inv: &mut FsFr = &mut prod[len - 1].eucl_inverse();

    i = len - 1;
    while i > 0 {
        out[i] = prod[i - 1].mul(inv);
        *inv = a[i].mul(inv);
        i -= 1;
    }
    out[0] = *inv;
}

pub fn bytes_to_bls_field_rust(bytes: &[u8; 32usize]) -> Result<FsFr, u8> {
    FsFr::from_scalar(*bytes)
}

pub fn vector_lincomb(vectors: &[Vec<FsFr>], scalars: &[FsFr]) -> Vec<FsFr> {
    let mut tmp: FsFr;
    let mut out: Vec<FsFr> = vec![FsFr::zero(); vectors[0].len()];
    for (v, s) in vectors.iter().zip(scalars.iter()) {
        for (i, x) in v.iter().enumerate() {
            tmp = x.mul(s);
            out[i] = out[i].add(&tmp);
        }
    }
    out
}

pub fn bytes_from_bls_field(fr: &FsFr) -> [u8; 32usize] {
    fr.to_scalar()
}

pub fn g1_lincomb(points: &[FsG1], scalars: &[FsFr], length: usize) -> FsG1 {
    let mut out = FsG1::default();
    g1_linear_combination(&mut out, points, scalars, length);
    out
}

pub fn blob_to_kzg_commitment_rust(blob: &[FsFr], s: &FsKZGSettings) -> FsG1 {
    let p = blob_to_polynomial_rust(blob);
    poly_to_kzg_commitment(&p, s)
}

pub fn verify_kzg_proof_rust(
    polynomial_kzg: &FsG1,
    z: &FsFr,
    y: &FsFr,
    kzg_proof: &FsG1,
    s: &FsKZGSettings,
) -> bool {
    s.check_proof_single(polynomial_kzg, kzg_proof, z, y)
        .unwrap_or(false)
}

pub fn verify_kzg_proof_batch(
    commitments_g1: &[FsG1],
    evaluation_challenges_fr: &[FsFr],
    ys_fr: &[FsFr],
    proofs_g1: &[FsG1],
    ts: &FsKZGSettings,
) -> bool {
    let n = commitments_g1.len();
    let mut c_minus_y: Vec<FsG1> = Vec::new();
    let mut r_times_z: Vec<FsFr> = Vec::new();

    // Compute the random lincomb challenges
    let r_powers = compute_r_powers(commitments_g1, evaluation_challenges_fr, ys_fr, proofs_g1);

    // Compute \sum r^i * Proof_i
    let proof_lincomb = g1_lincomb(proofs_g1, &r_powers, n);

    for i in 0..n {
        // Get [y_i]
        let ys_encrypted = FsG1::generator().mul(&ys_fr[i]);
        // Get C_i - [y_i]
        c_minus_y.push(commitments_g1[i].sub(&ys_encrypted));
        // Get r^i * z_i
        r_times_z.push(r_powers[i].mul(&evaluation_challenges_fr[i]));
    }

    // Get \sum r^i z_i Proof_i
    let proof_z_lincomb = g1_lincomb(proofs_g1, &r_times_z, n);
    // Get \sum r^i (C_i - [y_i])
    let mut c_minus_y_lincomb = g1_lincomb(&c_minus_y, &r_powers, n);

    // Get C_minus_y_lincomb + proof_z_lincomb
    let rhs_g1 = c_minus_y_lincomb.add_or_dbl(&proof_z_lincomb);

    // Do the pairing check!
    pairings_verify(&proof_lincomb, &ts.secret_g2[1], &rhs_g1, &FsG2::generator())
}

pub fn compute_kzg_proof_rust(polynomial: &FsPoly, z: &FsFr, s: &FsKZGSettings) -> FsG1 {
    let y: FsFr = evaluate_polynomial_in_evaluation_form_rust(polynomial, z, s);

    let mut tmp: FsFr;
    let mut roots_of_unity: Vec<FsFr> = s.fs.expanded_roots_of_unity.clone();
    reverse_bit_order(&mut roots_of_unity);

    let mut m: usize = 0;
    let mut q: FsPoly = FsPoly::new(FIELD_ELEMENTS_PER_BLOB).unwrap();

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

    fr_batch_inv(&mut inverses, &inverses_in, FIELD_ELEMENTS_PER_BLOB);

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        q.coeffs[i] = q.coeffs[i].mul(&inverses[i]);
    }

    if m > 0 { // ω_m == x
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

    g1_lincomb(&s.secret_g1, &q.coeffs, FIELD_ELEMENTS_PER_BLOB)
}

pub fn evaluate_polynomial_in_evaluation_form_rust(
    p: &FsPoly,
    x: &FsFr,
    s: &FsKZGSettings,
) -> FsFr {
    let mut roots_of_unity: Vec<FsFr> = s.fs.expanded_roots_of_unity.clone();
    let mut inverses_in: Vec<FsFr> = vec![FsFr::default(); p.len()];
    let mut inverses: Vec<FsFr> = vec![FsFr::default(); p.len()];

    reverse_bit_order(&mut roots_of_unity);

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

fn bytes_of_uint64(out: &mut [u8], mut n: u64) {
    for byte in out.iter_mut().take(8) {
        *byte = (n & 0xff) as u8;
        n >>= 8;
    }
}

fn hash(x: &[u8]) -> [u8; 32] {
    Sha256::digest(x).into()
}

pub fn hash_to_bls_field(x: &[u8; 32]) -> FsFr {
    let mut tmp = blst_scalar::default();
    let mut out = blst_fr::default();
    unsafe {
        blst_scalar_from_lendian(&mut tmp, x.as_ptr());
        blst_fr_from_scalar(&mut out, &tmp);
    }
    FsFr(out)
}

pub fn compute_challenge(blob: &[FsFr], commitment: &FsG1) -> FsFr {
    let mut bytes: Vec<u8> = vec![0; CHALLENGE_INPUT_SIZE];

    // Copy domain separator
    bytes[..16].copy_from_slice(&FIAT_SHAMIR_PROTOCOL_DOMAIN);
    bytes_of_uint64(&mut bytes[16..24], FIELD_ELEMENTS_PER_BLOB as u64);
    // Set all other bytes of this 16-byte (little-endian) field to zero
    bytes_of_uint64(&mut bytes[24..32], 0);

    // Copy blob
    for i in 0..blob.len() {
        let v = bytes_from_bls_field(&blob[i]);
        bytes[
            (32 + i * BYTES_PER_FIELD_ELEMENT)
                ..
            (32 + (i + 1) * BYTES_PER_FIELD_ELEMENT)]
            .copy_from_slice(&v);
    }

    // Copy commitment
    let v = bytes_from_g1_rust(&commitment);
    for i in 0..v.len() {
        bytes[32 + BYTES_PER_BLOB + i] = v[i];
    }

    // Now let's create the challenge!

    let hashed_data: [u8; 32] = hash(&bytes);
    let mut hash_input = [0u8; 33];

    hash_input[..32].copy_from_slice(&hashed_data);
    hash_input[32] = 0x0;

    hash_input[32] = 0x1;
    let eval_challenge = hash(&hash_input);

    hash_to_bls_field(&eval_challenge)
}

pub fn compute_r_powers(
    commitments_g1: &[FsG1],
    evaluation_challenges_fr: &[FsFr],
    ys_fr: &[FsFr],
    proofs_g1: &[FsG1],
) -> Vec<FsFr> {
    let n = commitments_g1.len();
    let input_size = 32 +
        n * (BYTES_PER_COMMITMENT +
            2 * BYTES_PER_FIELD_ELEMENT + BYTES_PER_PROOF);

    #[allow(unused_assignments)] let mut offset = 0;
    let mut bytes: Vec<u8> = vec![0; input_size];

    // Copy domain separator
    bytes[..16].copy_from_slice(&RANDOM_CHALLENGE_KZG_BATCH_DOMAIN);
    bytes_of_uint64(&mut bytes[16..24], FIELD_ELEMENTS_PER_BLOB as u64);
    bytes_of_uint64(&mut bytes[24..32], n as u64);
    offset = 32;

    for i in 0..n {
        // Copy commitment
        let v = bytes_from_g1_rust(&commitments_g1[i]);
        for j in 0..v.len() {
            bytes[offset + j] = v[j];
        }
        offset += BYTES_PER_COMMITMENT;

        // Copy evaluation challenge
        let v = bytes_from_bls_field(&evaluation_challenges_fr[i]);
        for j in 0..v.len() {
            bytes[offset + j] = v[j];
        }
        offset += BYTES_PER_FIELD_ELEMENT;

        // Copy polynomial's evaluation value
        let v = bytes_from_bls_field(&ys_fr[i]);
        for j in 0..v.len() {
            bytes[offset + j] = v[j];
        }
        offset += BYTES_PER_FIELD_ELEMENT;

        // Copy proof
        let v = bytes_from_g1_rust(&proofs_g1[i]);
        for j in 0..v.len() {
            bytes[offset + j] = v[j];
        }
        offset += BYTES_PER_PROOF;
    }

    // Make sure we wrote the entire buffer
    assert_eq!(offset, input_size);

    // Now let's create the challenge!

    let hashed_data: [u8; 32] = hash(&bytes);
    let mut hash_input = [0u8; 33];

    hash_input[..32].copy_from_slice(&hashed_data);
    hash_input[32] = 0x0;

    hash_input[32] = 0x1;
    let eval_challenge = hash(&hash_input);

    let r = hash_to_bls_field(&eval_challenge);
    compute_powers(&r, n)
}

fn blob_to_polynomial_rust(blob: &[FsFr]) -> FsPoly {
    let mut p: FsPoly = FsPoly::new(FIELD_ELEMENTS_PER_BLOB).unwrap();
    p.coeffs = blob.to_vec();
    p
}

fn poly_to_kzg_commitment(p: &FsPoly, s: &FsKZGSettings) -> FsG1 {
    g1_lincomb(&s.secret_g1, &p.coeffs, FIELD_ELEMENTS_PER_BLOB)
}

pub fn compute_blob_kzg_proof_rust(blob: &[FsFr], ts: &FsKZGSettings) -> FsG1 {
    let polynomial = blob_to_polynomial_rust(blob);
    let commitment_g1 = poly_to_kzg_commitment(&polynomial, ts);
    let evaluation_challenge_fr = compute_challenge(blob, &commitment_g1);
    compute_kzg_proof_rust(&polynomial, &evaluation_challenge_fr, ts)
}

pub fn verify_blob_kzg_proof_rust(
    blob: &[FsFr],
    commitment_g1: &FsG1,
    proof_g1: &FsG1,
    ts: &FsKZGSettings,
) -> bool {
    let polynomial = blob_to_polynomial_rust(blob);
    let evaluation_challenge_fr = compute_challenge(blob, &commitment_g1);
    let y_fr = evaluate_polynomial_in_evaluation_form_rust(&polynomial, &evaluation_challenge_fr, ts);
    verify_kzg_proof_rust(&commitment_g1, &evaluation_challenge_fr, &y_fr, proof_g1, ts)
}

pub fn verify_blob_kzg_proof_batch_rust(
    blobs: &[Vec<FsFr>],
    commitments_g1: &[FsG1],
    proofs_g1: &[FsG1],
    ts: &FsKZGSettings,
) -> bool {
    // Exit early if we are given zero blobs
    if blobs.len() == 0 {
        return true
    }

    let mut evaluation_challenges_fr: Vec<FsFr> = Vec::new();
    let mut ys_fr: Vec<FsFr> = Vec::new();

    for i in 0..blobs.len() {
        let polynomial = blob_to_polynomial_rust(&blobs[i]);
        let evaluation_challenge_fr = compute_challenge(&blobs[i], &commitments_g1[i]);
        let y_fr = evaluate_polynomial_in_evaluation_form_rust(&polynomial, &evaluation_challenge_fr, ts);

        evaluation_challenges_fr.push(evaluation_challenge_fr);
        ys_fr.push(y_fr);
    }

    verify_kzg_proof_batch(commitments_g1, &evaluation_challenges_fr, &ys_fr, proofs_g1, ts)
}

pub const C_KZG_RET_C_KZG_OK: C_KZG_RET = 0;
pub const C_KZG_RET_C_KZG_BADARGS: C_KZG_RET = 1;
pub const C_KZG_RET_C_KZG_ERROR: C_KZG_RET = 2;
pub const C_KZG_RET_C_KZG_MALLOC: C_KZG_RET = 3;
pub type C_KZG_RET = ::std::os::raw::c_uint;

pub const BYTES_PER_BLOB: usize = 32 * FIELD_ELEMENTS_PER_BLOB;
pub const CHALLENGE_INPUT_SIZE: usize = 32 + BYTES_PER_BLOB + 48;
pub const BYTES_PER_COMMITMENT: usize = 48;

#[repr(C)]
pub struct Bytes32 {
    pub bytes: [u8; 32],
}

#[repr(C)]
pub struct Bytes48 {
    pub bytes: [u8; 48],
}

#[repr(C)]
pub struct Blob {
    pub bytes: [u8; BYTES_PER_BLOB],
}

#[repr(C)]
pub struct KZGCommitment {
    pub bytes: [u8; 48],
}

#[repr(C)]
pub struct KZGProof {
    pub bytes: [u8; 48],
}

#[repr(C)]
pub struct CFsFFTSettings {
    pub max_width: u64,
    pub expanded_roots_of_unity: *mut blst_fr,
    pub reverse_roots_of_unity: *mut blst_fr,
    pub roots_of_unity: *mut blst_fr,
}

#[repr(C)]
pub struct CFsKzgSettings {
    pub fs: *const CFsFFTSettings,
    pub g1_values: *mut blst_p1, // G1
    pub g2_values: *mut blst_p2, // G2
}

#[repr(C)]
pub struct CFsPoly {
    pub evals: [blst_fr; FIELD_ELEMENTS_PER_BLOB],
}

fn fft_settings_to_rust(c_settings: *const CFsFFTSettings) -> FsFFTSettings {
    let settings = unsafe { &*c_settings };
    let mut first_root = unsafe { FsFr(*(settings.expanded_roots_of_unity.add(1))) };
    let first_root_arr = [first_root; 1];
    first_root = first_root_arr[0];

    let res = FsFFTSettings {
        max_width: settings.max_width as usize,
        root_of_unity: first_root,
        expanded_roots_of_unity: unsafe {
            std::slice::from_raw_parts(
                settings.expanded_roots_of_unity,
                (settings.max_width + 1) as usize,
            )
            .iter()
            .map(|r| FsFr(*r))
            .collect::<Vec<FsFr>>()
        },
        reverse_roots_of_unity: unsafe {
            std::slice::from_raw_parts(
                settings.reverse_roots_of_unity,
                (settings.max_width + 1) as usize,
            )
            .iter()
            .map(|r| FsFr(*r))
            .collect::<Vec<FsFr>>()
        },
    };

    res
}

fn fft_settings_to_c(rust_settings: &FsFFTSettings) -> *const CFsFFTSettings {
    let mut roots_of_unity: Vec<FsFr> = rust_settings.expanded_roots_of_unity.clone();
    reverse_bit_order(&mut roots_of_unity);
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
    let roots_of_unity = Box::new(roots_of_unity.iter().map(|r| r.0).collect::<Vec<blst_fr>>());

    let b = Box::new(CFsFFTSettings {
        max_width: rust_settings.max_width as u64,
        expanded_roots_of_unity: unsafe { (*Box::into_raw(expanded_roots_of_unity)).as_mut_ptr() },
        reverse_roots_of_unity: unsafe { (*Box::into_raw(reverse_roots_of_unity)).as_mut_ptr() },
        roots_of_unity: unsafe { (*Box::into_raw(roots_of_unity)).as_mut_ptr() },
    });
    Box::into_raw(b)
}

fn kzg_settings_to_rust(c_settings: &CFsKzgSettings) -> FsKZGSettings {
    let length = unsafe { (*c_settings.fs).max_width as usize };
    let secret_g1 = unsafe {
        std::slice::from_raw_parts(c_settings.g1_values, length)
            .iter()
            .map(|r| FsG1(*r))
            .collect::<Vec<FsG1>>()
    };
    let res = FsKZGSettings {
        fs: fft_settings_to_rust(c_settings.fs),
        secret_g1,
        secret_g2: unsafe {
            std::slice::from_raw_parts(c_settings.g2_values, 65)
                .iter()
                .map(|r| FsG2(*r))
                .collect::<Vec<FsG2>>()
        },
    };
    res
}

fn kzg_settings_to_c(rust_settings: &FsKZGSettings) -> CFsKzgSettings {
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

    CFsKzgSettings {
        fs: fft_settings_to_c(&rust_settings.fs),
        g1_values: unsafe { (*v).as_mut_ptr() },
        g2_values: stat_ref.as_mut_ptr(),
    }
}

fn poly_to_rust(c_poly: &CFsPoly) -> FsPoly {
    let c_poly_coeffs = c_poly.evals;
    let mut poly_rust = FsPoly::new(c_poly_coeffs.len()).unwrap();
    for (pos, e) in c_poly_coeffs.iter().enumerate() {
        poly_rust.set_coeff_at(pos, &FsFr(*e));
    }
    poly_rust
}

unsafe fn deserialize_blob(blob: *const Blob) -> Result<Vec<FsFr>, C_KZG_RET> {
    (*blob)
        .bytes
        .chunks(32)
        .map(|x| {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(x);
            let result = bytes_to_bls_field_rust(&bytes);
            if result.is_ok() {
                Ok(result.unwrap())
            } else {
                Err(result.err().unwrap() as C_KZG_RET)
            }
        })
        .collect::<Result<Vec<FsFr>, C_KZG_RET>>()
}

/// # Safety
///
/// This function should not be called before the horsemen are ready.
#[no_mangle]
pub unsafe extern "C" fn blob_to_kzg_commitment(
    out: *mut KZGCommitment,
    blob: *const Blob,
    s: &CFsKzgSettings,
) -> C_KZG_RET {
    let deserialized_blob = deserialize_blob(blob);
    if let Ok(blob_) = deserialized_blob {
        let tmp = blob_to_kzg_commitment_rust(&blob_, &kzg_settings_to_rust(s));
        (*out).bytes = bytes_from_g1_rust(&tmp);
        C_KZG_RET_C_KZG_OK
    } else {
        deserialized_blob.err().unwrap()
    }
}

#[no_mangle]
/// # Safety
///
/// This function should not be called before the horsemen are ready.
pub unsafe extern "C" fn load_trusted_setup(
    out: *mut CFsKzgSettings,
    g1_bytes: *const u8,
    n1: usize,
    g2_bytes: *const u8,
    n2: usize,
) -> C_KZG_RET {
    let g1_bytes = std::slice::from_raw_parts(g1_bytes, n1 * 48);
    let g2_bytes = std::slice::from_raw_parts(g2_bytes, n2 * 96);
    let settings = load_trusted_setup_rust(g1_bytes, n1, g2_bytes, n2);
    *out = kzg_settings_to_c(&settings);
    C_KZG_RET_C_KZG_OK
}

// getting *FILE seems impossible
// https://stackoverflow.com/questions/4862327/is-there-a-way-to-get-the-filename-from-a-file
/// # Safety
///
/// This function should not be called before the horsemen are ready.
#[no_mangle]
pub unsafe extern "C" fn load_trusted_setup_file(
    out: *mut CFsKzgSettings,
    inp: *mut FILE,
) -> C_KZG_RET {
    let mut buf: [c_char; 100] = [0; 100];
    let result = fgets(buf.as_mut_ptr(), 100, inp);
    if result.is_null()
        || strtoul(buf.as_ptr(), null_mut(), 10) != FIELD_ELEMENTS_PER_BLOB as c_ulong
    {
        return C_KZG_RET_C_KZG_BADARGS;
    }
    let result: *mut c_char = fgets(buf.as_mut_ptr(), 100, inp);
    if result.is_null() || strtoul(buf.as_ptr(), null_mut(), 10) != 65 {
        return C_KZG_RET_C_KZG_BADARGS;
    }

    let mut g2_bytes: [u8; 65 * 96] = [0; 65 * 96];
    let mut g1_bytes: [u8; FIELD_ELEMENTS_PER_BLOB * 48] = [0; FIELD_ELEMENTS_PER_BLOB * 48];

    let mut i: usize = 0;
    while i < FIELD_ELEMENTS_PER_BLOB * 48 {
        let c1 = fgetc(inp) as c_char;
        if c1 == '\n' as c_char {
            continue;
        }
        let c2 = fgetc(inp) as c_char;

        if c1 == EOF as c_char || c2 == EOF as c_char {
            return 1;
        }
        g1_bytes[i] = strtoul([c1, c2].as_ptr(), null_mut(), 16) as u8;
        i += 1;
    }

    i = 0;
    while i < 65 * 96 {
        let c1 = fgetc(inp) as c_char;
        if c1 == '\n' as c_char {
            continue;
        }
        let c2 = fgetc(inp) as c_char;

        if c1 == EOF as c_char || c2 == EOF as c_char {
            return 1;
        }
        g2_bytes[i] = strtoul([c1, c2].as_ptr(), null_mut(), 16) as u8;
        i += 1;
    }

    let settings = load_trusted_setup_rust(&g1_bytes, FIELD_ELEMENTS_PER_BLOB, &g2_bytes, 65);
    *out = kzg_settings_to_c(&settings);
    C_KZG_RET_C_KZG_OK
}

#[no_mangle]
/// # Safety
///
/// This function should not be called before the horsemen are ready
pub unsafe extern "C" fn compute_blob_kzg_proof(
    out: *mut KZGProof,
    blob: *const Blob,
    s: &CFsKzgSettings,
) -> C_KZG_RET {
    let deserialized_blob = deserialize_blob(blob);
    if deserialized_blob.is_err() {
        return deserialized_blob.err().unwrap();
    }
    let commitment_g1 = compute_blob_kzg_proof_rust(
        &deserialized_blob.unwrap(),
        &kzg_settings_to_rust(s));
    (*out).bytes = bytes_from_g1_rust(&commitment_g1);
    C_KZG_RET_C_KZG_OK
}

#[no_mangle]
/// # Safety
///
/// This function should not be called before the horsemen are ready
pub unsafe extern "C" fn free_trusted_setup(s: *mut CFsKzgSettings) {
    let max_width = (*(*s).fs).max_width as usize;
    let rev = Box::from_raw(std::slice::from_raw_parts_mut(
        (*(*s).fs).reverse_roots_of_unity,
        max_width,
    ));
    drop(rev);
    let exp = Box::from_raw(std::slice::from_raw_parts_mut(
        (*(*s).fs).expanded_roots_of_unity,
        max_width,
    ));
    drop(exp);
    let roots = Box::from_raw(std::slice::from_raw_parts_mut(
        (*(*s).fs).roots_of_unity,
        max_width,
    ));
    drop(roots);
    let g1 = Box::from_raw(std::slice::from_raw_parts_mut((*s).g1_values, max_width));
    drop(g1);
    let g2 = Box::from_raw(std::slice::from_raw_parts_mut((*s).g2_values, 65));
    drop(g2);
}

#[no_mangle]
/// # Safety
///
/// This function should not be called before the horsemen are ready
pub unsafe extern "C" fn verify_kzg_proof(
    out: *mut bool,
    commitment_bytes: *const Bytes48,
    z_bytes: *const Bytes32,
    y_bytes: *const Bytes32,
    proof_bytes: *const Bytes48,
    s: &CFsKzgSettings,
) -> C_KZG_RET {
    let frz = bytes_to_bls_field_rust(&(*z_bytes).bytes);
    let fry = bytes_to_bls_field_rust(&(*y_bytes).bytes);
    let g1commitment = bytes_to_g1_rust(&(*commitment_bytes).bytes);
    let g1proof = bytes_to_g1_rust(&(*proof_bytes).bytes);

    if frz.is_err() || fry.is_err() || g1commitment.is_err() || g1proof.is_err() {
        return C_KZG_RET_C_KZG_BADARGS;
    }

    *out = verify_kzg_proof_rust(
        &g1commitment.unwrap(),
        &frz.unwrap(),
        &fry.unwrap(),
        &g1proof.unwrap(),
        &kzg_settings_to_rust(s),
    );
    C_KZG_RET_C_KZG_OK
}

#[no_mangle]
/// # Safety
///
/// This function should not be called before the horsemen are ready
pub unsafe extern "C" fn verify_blob_kzg_proof(
    ok: *mut bool,
    blob: *const Blob,
    commitment_bytes: *const Bytes48,
    proof_bytes: *const Bytes48,
    s: &CFsKzgSettings,
) -> C_KZG_RET {
    let deserialized_blob = deserialize_blob(blob);
    if deserialized_blob.is_err() {
        return deserialized_blob.err().unwrap();
    }

    let commitment_g1 = bytes_to_g1_rust(&(*commitment_bytes).bytes);
    let proof_g1 = bytes_to_g1_rust(&(*proof_bytes).bytes);
    if commitment_g1.is_err() || proof_g1.is_err() {
        return C_KZG_RET_C_KZG_BADARGS;
    }

    *ok = verify_blob_kzg_proof_rust(
        &deserialized_blob.unwrap(),
        &commitment_g1.unwrap(),
        &proof_g1.unwrap(),
        &kzg_settings_to_rust(s));
    C_KZG_RET_C_KZG_OK
}

#[no_mangle]
/// # Safety
///
/// This function should not be called before the horsemen are ready
pub unsafe extern "C" fn verify_blob_kzg_proof_batch(
    ok: *mut bool,
    blobs: *const Blob,
    commitments_bytes: *const Bytes48,
    proofs_bytes: *const Bytes48,
    n: usize,
    s: &CFsKzgSettings,
) -> C_KZG_RET {
    let mut deserialized_blobs: Vec<Vec<FsFr>> = Vec::new();
    let mut commitments_g1: Vec<FsG1> = Vec::new();
    let mut proofs_g1: Vec<FsG1> = Vec::new();

    let raw_blobs = std::slice::from_raw_parts(blobs, n);
    let raw_commitments = std::slice::from_raw_parts(commitments_bytes, n);
    let raw_proofs = std::slice::from_raw_parts(proofs_bytes, n);

    for i in 0..n {
        let deserialized_blob = deserialize_blob(&raw_blobs[i]);
        if deserialized_blob.is_err() {
            return deserialized_blob.err().unwrap();
        }

        let commitment_g1 = bytes_to_g1_rust(&raw_commitments[i].bytes);
        let proof_g1 = bytes_to_g1_rust(&raw_proofs[i].bytes);
        if commitment_g1.is_err() || proof_g1.is_err() {
            return C_KZG_RET_C_KZG_BADARGS;
        }

        deserialized_blobs.push(deserialized_blob.unwrap());
        commitments_g1.push(commitment_g1.unwrap());
        proofs_g1.push(proof_g1.unwrap());
    }

    *ok = verify_blob_kzg_proof_batch_rust(
        &deserialized_blobs,
        &commitments_g1,
        &proofs_g1,
        &kzg_settings_to_rust(s));
    C_KZG_RET_C_KZG_OK
}

#[no_mangle]
/// # Safety
///
/// This function should not be called before the horsemen are ready
pub unsafe extern "C" fn compute_kzg_proof(
    out: *mut KZGProof,
    blob: *const Blob,
    z_bytes: *const Bytes32,
    s: &CFsKzgSettings,
) -> C_KZG_RET {
    let deserialized_blob = deserialize_blob(blob);
    if deserialized_blob.is_err() {
        return deserialized_blob.err().unwrap();
    }
    let poly = blob_to_polynomial_rust(&deserialized_blob.unwrap());
    let frz = bytes_to_bls_field_rust(&(*z_bytes).bytes);
    if frz.is_err() {
        return frz.err().unwrap() as C_KZG_RET;
    }
    let tmp = compute_kzg_proof_rust(&poly, &frz.unwrap(), &kzg_settings_to_rust(s));
    (*out).bytes = bytes_from_g1_rust(&tmp);
    C_KZG_RET_C_KZG_OK
}

#[no_mangle]
/// # Safety
///
/// This function should not be called before the horsemen are ready
pub unsafe extern "C" fn evaluate_polynomial_in_evaluation_form(
    out: *mut blst_fr,
    p: &CFsPoly,
    x: &blst_fr,
    s: &CFsKzgSettings,
) -> C_KZG_RET {
    *out = evaluate_polynomial_in_evaluation_form_rust(
        &poly_to_rust(p),
        &FsFr(*x),
        &kzg_settings_to_rust(s),
    )
    .0;
    C_KZG_RET_C_KZG_OK
}

#[no_mangle]
/// # Safety
///
/// This function should not be called before the horsemen are ready
pub unsafe extern "C" fn bytes_to_bls_field(
    out: *mut blst_fr,
    b: &Bytes32
) -> C_KZG_RET {
    let fr = bytes_to_bls_field_rust(&b.bytes);
    if fr.is_err() {
        return fr.err().unwrap() as C_KZG_RET
    }
    *out = fr.unwrap().0;
    C_KZG_RET_C_KZG_OK
}

#[no_mangle]
/// # Safety
///
/// This function should not be called before the horsemen are ready
pub unsafe extern "C" fn blob_to_polynomial(
    p: *mut CFsPoly,
    blob: *const Blob
) -> C_KZG_RET {
    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        let start = i * BYTES_PER_FIELD_ELEMENT;
        let bytes_array: [u8; BYTES_PER_FIELD_ELEMENT] = (*blob).bytes
            [start..(start + BYTES_PER_FIELD_ELEMENT)]
            .try_into()
            .unwrap();
        let bytes = Bytes32 { bytes: bytes_array };
        let fr = bytes_to_bls_field_rust(&bytes.bytes);
        if fr.is_err() {
            return fr.err().unwrap() as C_KZG_RET
        }
        (*p).evals[i] = fr.unwrap().0;
    }
    C_KZG_RET_C_KZG_OK
}
