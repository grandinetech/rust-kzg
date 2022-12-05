use std::convert::TryInto;
use std::fs::File;
use std::io::Read;

use blst::{
    blst_p1, blst_p1_affine, blst_p1_compress, blst_p1_from_affine, blst_p1_uncompress, blst_p2,
    blst_p2_affine, blst_p2_from_affine, blst_p2_uncompress, BLST_ERROR,
};
use kzg::{FFTSettings, Fr, KZGSettings, Poly, FFTG1, G1};
use sha2::{Digest, Sha256};

#[cfg(feature = "parallel")]
use rayon::iter::{IntoParallelRefIterator as _, ParallelIterator as _};

use crate::consts::{
    BYTES_PER_FIELD_ELEMENT, FIAT_SHAMIR_PROTOCOL_DOMAIN, FIELD_ELEMENTS_PER_BLOB,
};
use crate::types::fft_settings::FsFFTSettings;
use crate::types::fr::FsFr;
use crate::types::g1::FsG1;

use crate::kzg_proofs::g1_linear_combination;
use crate::types::g2::FsG2;
use crate::types::kzg_settings::FsKZGSettings;
use crate::types::poly::FsPoly;
use crate::utils::reverse_bit_order;

pub fn bytes_to_g1(bytes: &[u8; 48usize]) -> FsG1 {
    let mut tmp = blst_p1_affine::default();
    let mut g1 = blst_p1::default();
    unsafe {
        if blst_p1_uncompress(&mut tmp, bytes.as_ptr()) != BLST_ERROR::BLST_SUCCESS {
            panic!("blst_p1_uncompress failed");
        }
        blst_p1_from_affine(&mut g1, &tmp);
    }
    FsG1(g1)
}

pub fn bytes_from_g1(g1: &FsG1) -> [u8; 48usize] {
    let mut out: [u8; 48usize] = [0; 48];
    unsafe {
        blst_p1_compress(out.as_mut_ptr(), &g1.0);
    }
    out
}

pub fn load_trusted_setup(filepath: &str) -> FsKZGSettings {
    let mut file = File::open(filepath).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    let mut lines = contents.lines();
    let length = lines.next().unwrap().parse::<usize>().unwrap();
    let n2 = lines.next().unwrap().parse::<usize>().unwrap();

    let mut g2_values: Vec<FsG2> = Vec::new();

    let mut g1_projectives: Vec<FsG1> = Vec::new();

    for _ in 0..length {
        let line = lines.next().unwrap();
        assert!(line.len() == 96);
        let bytes_array: [u8; 48] = (0..line.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&line[i..i + 2], 16).unwrap())
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap();
        g1_projectives.push(bytes_to_g1(&bytes_array));
    }

    for _ in 0..n2 {
        let line = lines.next().unwrap();
        assert!(line.len() == 192);
        let bytes = (0..line.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&line[i..i + 2], 16).unwrap())
            .collect::<Vec<u8>>();
        let mut tmp = blst_p2_affine::default();
        let mut g2 = blst_p2::default();
        unsafe {
            if blst_p2_uncompress(&mut tmp, bytes.as_ptr()) != BLST_ERROR::BLST_SUCCESS {
                panic!("blst_p2_uncompress failed");
            }
            blst_p2_from_affine(&mut g2, &tmp);
        }
        g2_values.push(FsG2(g2));
    }

    let mut max_scale: usize = 0;
    while (1 << max_scale) < length {
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

pub fn bytes_to_bls_field(bytes: &[u8; 32usize]) -> FsFr {
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

pub fn g1_lincomb(points: &[FsG1], scalars: &[FsFr]) -> FsG1 {
    assert!(points.len() == scalars.len());
    let mut out = FsG1::default();
    g1_linear_combination(&mut out, points, scalars, points.len());
    out
}

pub fn blob_to_kzg_commitment(blob: &[FsFr], s: &FsKZGSettings) -> FsG1 {
    g1_lincomb(&s.secret_g1, blob)
}

pub fn verify_kzg_proof(
    polynomial_kzg: &FsG1,
    z: &FsFr,
    y: &FsFr,
    kzg_proof: &FsG1,
    s: &FsKZGSettings,
) -> bool {
    s.check_proof_single(polynomial_kzg, kzg_proof, z, y)
        .unwrap_or(false)
}

pub fn compute_kzg_proof(p: &FsPoly, x: &FsFr, s: &FsKZGSettings) -> FsG1 {
    assert!(p.len() <= s.secret_g1.len());

    let y: FsFr = evaluate_polynomial_in_evaluation_form(p, x, s);

    let mut tmp: FsFr;
    let mut roots_of_unity: Vec<FsFr> = s.fs.expanded_roots_of_unity.clone();

    reverse_bit_order(&mut roots_of_unity);
    let mut i: usize = 0;
    let mut m: usize = 0;

    let mut q: FsPoly = FsPoly::new(p.len()).unwrap();

    let mut inverses_in: Vec<FsFr> = vec![FsFr::default(); p.len()];
    let mut inverses: Vec<FsFr> = vec![FsFr::default(); p.len()];

    while i < q.len() {
        if x.equals(&roots_of_unity[i]) {
            m = i + 1;
            continue;
        }
        // (p_i - y) / (ω_i - x)
        q.coeffs[i] = p.coeffs[i].sub(&y);
        inverses_in[i] = roots_of_unity[i].sub(x);
        i += 1;
    }

    fr_batch_inv(&mut inverses, &inverses_in, q.len());

    i = 0;
    while i < q.len() {
        q.coeffs[i] = q.coeffs[i].mul(&inverses[i]);
        i += 1;
    }

    if m > 0 {
        // ω_m == x
        q.coeffs[m] = FsFr::zero();
        m -= 1;
        i = 0;
        while i < q.coeffs.len() {
            if i == m {
                continue;
            }
            // (p_i - y) * ω_i / (x * (x - ω_i))
            tmp = x.sub(&roots_of_unity[i]);
            inverses_in[i] = tmp.mul(x);
            i += 1;
        }
        fr_batch_inv(&mut inverses, &inverses_in, q.coeffs.len());
        i = 0;
        while i < q.coeffs.len() {
            tmp = p.coeffs[i].sub(&y);
            tmp = tmp.mul(&roots_of_unity[i]);
            tmp = tmp.mul(&inverses[i]);
            q.coeffs[m] = q.coeffs[m].add(&tmp);
            i += 1;
        }
    }

    g1_lincomb(&s.secret_g1, &q.coeffs)
}

pub fn evaluate_polynomial_in_evaluation_form(p: &FsPoly, x: &FsFr, s: &FsKZGSettings) -> FsFr {
    let mut tmp: FsFr;

    let mut inverses_in: Vec<FsFr> = vec![FsFr::default(); p.len()];
    let mut inverses: Vec<FsFr> = vec![FsFr::default(); p.len()];
    let mut i: usize = 0;
    let mut roots_of_unity: Vec<FsFr> = s.fs.expanded_roots_of_unity.clone();

    reverse_bit_order(&mut roots_of_unity);

    while i < p.len() {
        if x.equals(&roots_of_unity[i]) {
            return p.get_coeff_at(i);
        }

        inverses_in[i] = x.sub(&roots_of_unity[i]);
        i += 1;
    }
    fr_batch_inv(&mut inverses, &inverses_in, p.len());

    let mut out = FsFr::zero();
    i = 0;
    while i < p.len() {
        tmp = inverses[i].mul(&roots_of_unity[i]);
        tmp = tmp.mul(&p.coeffs[i]);
        out = out.add(&tmp);
        i += 1;
    }
    tmp = FsFr::from_u64(p.len().try_into().unwrap());
    out = out.div(&tmp).unwrap();
    tmp = x.pow(p.len());
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

pub fn hash_to_bytes(polys: &[FsPoly], comms: &[FsG1], n: usize) -> [u8; 32] {
    let ni: usize = 32; // len(FIAT_SHAMIR_PROTOCOL_DOMAIN) + 8 + 8
    let np: usize = ni + n * FIELD_ELEMENTS_PER_BLOB * 32;

    let mut bytes: Vec<u8> = vec![0; np + n * 48];

    bytes[..16].copy_from_slice(&FIAT_SHAMIR_PROTOCOL_DOMAIN);

    bytes_of_uint64(&mut bytes[16..24], n.try_into().unwrap());
    bytes_of_uint64(
        &mut bytes[24..32],
        FIELD_ELEMENTS_PER_BLOB.try_into().unwrap(),
    );

    for i in 0..n {
        for j in 0..FIELD_ELEMENTS_PER_BLOB {
            let v = bytes_from_bls_field(&polys[i].get_coeff_at(j));
            for k in 0..32 {
                bytes[ni + i * BYTES_PER_FIELD_ELEMENT as usize + k] = v[k];
            }
        }
    }

    for i in 0..n {
        let v = bytes_from_g1(&comms[i]);
        for k in 0..48 {
            bytes[np + i * 48 + k] = v[k];
        }
    }

    hash(&bytes)
}

pub fn poly_lincomb(vectors: &[FsPoly], scalars: &[FsFr], n: usize) -> FsPoly {
    let mut tmp: FsFr;
    let mut out: FsPoly = FsPoly::new(FIELD_ELEMENTS_PER_BLOB).unwrap();
    for j in 0..FIELD_ELEMENTS_PER_BLOB {
        out.set_coeff_at(j, &FsFr::zero());
    }
    for i in 0..n {
        for j in 0..FIELD_ELEMENTS_PER_BLOB {
            tmp = scalars[i].mul(&vectors[i].get_coeff_at(j));
            out.set_coeff_at(j, &out.get_coeff_at(j).add(&tmp));
        }
    }
    out
}

pub fn compute_aggregated_poly_and_commitment(
    polys: &[FsPoly],
    kzg_commitments: &[FsG1],
    n: usize,
) -> (FsPoly, FsG1, FsFr) {
    let hash = hash_to_bytes(polys, kzg_commitments, n);
    let r = bytes_to_bls_field(&hash);

    let (r_powers, chal_out) = if n == 1 {
        (vec![r], r)
    } else {
        let r_powers = compute_powers(&r, n);
        let chal_out = r_powers[1].mul(&r_powers[n - 1]);
        (r_powers, chal_out)
    };

    let poly_out = poly_lincomb(polys, &r_powers, n);

    let comm_out = g1_lincomb(kzg_commitments, &r_powers);

    (poly_out, comm_out, chal_out)
}

fn poly_from_blob(p: &mut FsPoly, blob: &[FsFr]) {
    for (i, coeff) in blob.iter().enumerate() {
        p.set_coeff_at(i, coeff);
    }
}

fn poly_to_kzg_commitment(p: &FsPoly, s: &FsKZGSettings) -> FsG1 {
    g1_lincomb(&s.secret_g1, &p.coeffs)
}

#[cfg(feature = "parallel")]
pub fn compute_aggregate_kzg_proof(blobs: &[Vec<FsFr>], ts: &FsKZGSettings) -> FsG1 {
    if blobs.len() == 0 {
        return FsG1::identity();
    }

    let (polys, commitments): (Vec<_>, Vec<_>) = blobs
        .par_iter()
        .map(|blob| {
            let mut poly = FsPoly::new(FIELD_ELEMENTS_PER_BLOB).unwrap();
            poly_from_blob(&mut poly, blob);

            let commitment = poly_to_kzg_commitment(&poly, ts);
            (poly, commitment)
        })
        .unzip();

    let (aggregated_poly, _, evaluation_challenge) =
        compute_aggregated_poly_and_commitment(&polys, &commitments, blobs.len());
    compute_kzg_proof(&aggregated_poly, &evaluation_challenge, ts)
}

#[cfg(not(feature = "parallel"))]
pub fn compute_aggregate_kzg_proof(blobs: &[Vec<FsFr>], ts: &FsKZGSettings) -> FsG1 {
    let n = blobs.len();
    if n == 0 {
        return FsG1::identity();
    }
    let mut commitments: Vec<FsG1> = vec![FsG1::default(); n];
    let mut polys: Vec<FsPoly> = vec![FsPoly::new(FIELD_ELEMENTS_PER_BLOB).unwrap(); n];
    for i in 0..n {
        poly_from_blob(&mut polys[i], &blobs[i]);
        commitments[i] = poly_to_kzg_commitment(&polys[i], ts);
    }
    let (aggregated_poly, _, evaluation_challenge) =
        compute_aggregated_poly_and_commitment(&polys, &commitments, n);
    compute_kzg_proof(&aggregated_poly, &evaluation_challenge, ts)
}

pub fn verify_aggregate_kzg_proof(
    blobs: &[Vec<FsFr>],
    expected_kzg_commitments: &[FsG1],
    kzg_aggregated_proof: &FsG1,
    ts: &FsKZGSettings,
) -> bool {
    let mut polys: Vec<FsPoly> = vec![FsPoly::new(FIELD_ELEMENTS_PER_BLOB).unwrap(); blobs.len()];
    for i in 0..blobs.len() {
        poly_from_blob(&mut polys[i], &blobs[i]);
    }
    let (aggregated_poly, aggregated_poly_commitment, evaluation_challenge) =
        compute_aggregated_poly_and_commitment(&polys, expected_kzg_commitments, blobs.len());
    let y = evaluate_polynomial_in_evaluation_form(&aggregated_poly, &evaluation_challenge, ts);
    verify_kzg_proof(
        &aggregated_poly_commitment,
        &evaluation_challenge,
        &y,
        kzg_aggregated_proof,
        ts,
    )
}
