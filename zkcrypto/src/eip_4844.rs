use std::convert::{TryFrom, TryInto};
use std::fs::File;
use std::io::Read;

use crate::fk20::{reverse_bit_order};
use crate::poly::KzgPoly;
use crate::kzg_proofs::{ KZGSettings, check_proof_single};
use crate::kzg_types::{ZkG1Affine, ZkG1Projective, ZkG2Affine, ZkG2Projective};
use crate::zkfr::blsScalar;
use sha2::{Digest, Sha256};
use kzg::{G1, Poly, Fr, FFTSettings, FFTG1};

#[cfg(feature = "parallel")]
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
#[cfg(feature = "parallel")]
use rayon::iter::IntoParallelIterator;
use crate::curve::g1::G1Affine;
use crate::curve::g2::G2Affine;
use crate::curve::scalar::{sbb, Scalar};
use crate::fftsettings::ZkFFTSettings;

const MODULUS: Scalar = Scalar([
    0xffff_ffff_0000_0001,
    0x53bd_a402_fffe_5bfe,
    0x3339_d808_09a1_d805,
    0x73ed_a753_299d_7d48,
]);

const R2: Scalar = Scalar([
    0xc999_e990_f3f2_9c6d,
    0x2b6c_edcb_8792_5c23,
    0x05d3_1496_7254_398f,
    0x0748_d9d9_9f59_ff11,
]);

pub fn bytes_to_bls_field(bytes: &[u8; 32usize]) -> blsScalar {
    let mut tmp = Scalar([0, 0, 0, 0]);

    tmp.0[0] = u64::from_le_bytes(<[u8; 8]>::try_from(&bytes[0..8]).unwrap());
    tmp.0[1] = u64::from_le_bytes(<[u8; 8]>::try_from(&bytes[8..16]).unwrap());
    tmp.0[2] = u64::from_le_bytes(<[u8; 8]>::try_from(&bytes[16..24]).unwrap());
    tmp.0[3] = u64::from_le_bytes(<[u8; 8]>::try_from(&bytes[24..32]).unwrap());

    // Try to subtract the modulus
    let (_, borrow) = sbb(tmp.0[0], MODULUS.0[0], 0);
    let (_, borrow) = sbb(tmp.0[1], MODULUS.0[1], borrow);
    let (_, borrow) = sbb(tmp.0[2], MODULUS.0[2], borrow);
    let (_, borrow) = sbb(tmp.0[3], MODULUS.0[3], borrow);

    // If the element is smaller than MODULUS then the
    // subtraction will underflow, producing a borrow value
    // of 0xffff...ffff. Otherwise, it'll be zero.
    let is_some = (borrow as u8) & 1;

    // Convert to Montgomery form by computing
    // (a.R^0 * R^2) / R = a.R
    tmp *= &R2;
    tmp
}

pub fn bytes_from_bls_field(fr: &blsScalar) -> [u8; 32usize] {
    fr.to_bytes()
}

pub fn compute_powers(base: &blsScalar, num_powers: usize) -> Vec<blsScalar> {
    let mut powers: Vec<blsScalar> = vec![blsScalar::default(); num_powers];
    powers[0] = blsScalar::one();
    for i in 1..num_powers {
        powers[i] = powers[i - 1].mul(base);
    }
    powers
}

pub fn vector_lincomb(vectors: &[Vec<blsScalar>], scalars: &[blsScalar]) -> Vec<blsScalar> {
    let mut tmp: blsScalar;
    let mut out: Vec<blsScalar> = vec![blsScalar::zero(); vectors[0].len()];
    for (v, s) in vectors.iter().zip(scalars.iter()) {
        for (i, x) in v.iter().enumerate() {
            tmp = x.mul(s);
            out[i] = out[i].add(&tmp);
        }
    }
    out
}

pub fn g1_lincomb(points: &[ZkG1Projective], scalars: &[blsScalar]) -> ZkG1Projective {
    assert!(points.len() == scalars.len());
    let mut out = ZkG1Projective::default();
    g1_linear_combination(&mut out, points, scalars, points.len());
    out
}

pub fn g1_linear_combination(
    out: &mut ZkG1Projective,
    p: &[ZkG1Projective],
    coeffs: &[blsScalar],
    len: usize,
) {
    let mut tmp;
    *out = G1::identity();
    for i in 0..len {
        tmp = p[i].mul(&coeffs[i]);
        *out = out.add_or_dbl(&tmp);
    }
}

pub fn blob_to_kzg_commitment(blob: &[blsScalar], s: &KZGSettings) -> ZkG1Projective {
    g1_lincomb(&s.secret_g1, blob)
}

pub fn fr_batch_inv(out: &mut [blsScalar], a: &[blsScalar], len: usize) {
    let prod: &mut Vec<blsScalar> = &mut vec![blsScalar::default(); len];
    let mut i: usize = 1;

    prod[0] = a[0];

    while i < len {
        prod[i] = a[i].mul(&prod[i - 1]);
        i += 1;
    }

    let inv: &mut blsScalar = &mut prod[len - 1].eucl_inverse();

    i = len - 1;
    while i > 0 {
        out[i] = prod[i - 1].mul(inv);
        *inv = a[i].mul(inv);
        i -= 1;
    }
    out[0] = *inv;
}

pub fn evaluate_polynomial_in_evaluation_form(
    p: &KzgPoly,
    x: &blsScalar,
    s: &KZGSettings,
) -> blsScalar {
    let mut tmp: blsScalar;

    let mut inverses_in: Vec<blsScalar> = vec![blsScalar::default(); p.len()];
    let mut inverses: Vec<blsScalar> = vec![blsScalar::default(); p.len()];
    let mut i: usize = 0;
    let mut roots_of_unity: Vec<blsScalar> = s.fs.expanded_roots_of_unity.clone();

    reverse_bit_order(&mut roots_of_unity);

    while i < p.len() {
        if x.equals(&roots_of_unity[i]) {
            return p.get_coeff_at(i);
        }

        inverses_in[i] = x.sub(&roots_of_unity[i]);
        i += 1;
    }
    fr_batch_inv(&mut inverses, &inverses_in, p.len());

    let mut out = blsScalar::zero();
    i = 0;
    while i < p.len() {
        tmp = inverses[i].mul(&roots_of_unity[i]);
        tmp = tmp.mul(&p.coeffs[i]);
        out = out.add(&tmp);
        i += 1;
    }
    tmp = blsScalar::from_u64(p.len().try_into().unwrap());
    out = out.div(&tmp).unwrap();

    tmp = <blsScalar as Fr>::pow(x, p.len());
    tmp = tmp.sub(&blsScalar::one());
    out = out.mul(&tmp);
    out
}

pub fn bytes_to_g1(bytes: &[u8; 48usize]) -> ZkG1Projective {
    let affine: G1Affine = G1Affine::from_compressed(bytes).unwrap();
    ZkG1Projective::from(affine)
}

pub fn bytes_to_g2(bytes: &[u8; 96usize]) -> ZkG2Projective {
    let affine: G2Affine = G2Affine::from_compressed(bytes).unwrap();
    ZkG2Projective::from(affine)
}

pub fn bytes_from_g1(g1: &ZkG1Projective) -> [u8; 48usize] {
    let g1_affine = ZkG1Affine::from(g1);
    g1_affine.to_compressed()
}

pub fn bytes_from_g2(g2: &ZkG2Projective) -> [u8; 96usize] {
    let g2_affine = ZkG2Affine::from(g2);
    g2_affine.to_compressed()
}

pub fn load_trusted_setup(filepath: &str) -> KZGSettings {
    let mut file = File::open(filepath).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    let mut lines = contents.lines();
    let length = lines.next().unwrap().parse::<usize>().unwrap();
    let n2 = lines.next().unwrap().parse::<usize>().unwrap();

    let mut g1_projectives: Vec<ZkG1Projective> = Vec::new();
    let mut g2_values: Vec<ZkG2Projective> = Vec::new();

    for _ in 0..length {
        let line = lines.next().unwrap();
        assert_eq!(line.len(), 96);
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
        assert_eq!(line.len(), 192);
        let bytes_array: [u8; 96] = (0..line.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&line[i..i + 2], 16).unwrap())
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap();
        g2_values.push(bytes_to_g2(&bytes_array));
    }

    let mut max_scale: usize = 0;
    while (1 << max_scale) < length {
        max_scale += 1;
    }

    let fs = ZkFFTSettings::new(max_scale).unwrap();
    let mut g1_values = fs.fft_g1(&g1_projectives, true).unwrap();
    reverse_bit_order(&mut g1_values);

    KZGSettings {
        secret_g1: g1_values,
        secret_g2: g2_values,
        fs,
        length: 0,
    }
}

pub fn compute_kzg_proof(p: &KzgPoly, x: &blsScalar, s: &KZGSettings) -> ZkG1Projective {
    assert!(p.coeffs.len() <= s.secret_g1.len());

    let y = evaluate_polynomial_in_evaluation_form(p, x, s);

    let mut tmp: blsScalar;

    let mut roots_of_unity = s.fs.expanded_roots_of_unity.clone();

    reverse_bit_order(&mut roots_of_unity);

    let mut m = 0;

    let mut q = KzgPoly::new(p.coeffs.len()).unwrap();
    let plen = p.coeffs.len();
    let qlen = q.coeffs.len();

    let mut inverses_in: Vec<blsScalar> = vec![blsScalar::one(); plen];
    let mut inverses: Vec<blsScalar> = vec![blsScalar::one(); plen];

    for i in 0..qlen {
        if x == &roots_of_unity[i] {
            m = i + 1;
            continue;
        }
        q.coeffs[i] = p.coeffs[i] - y;
        inverses_in[i] = &roots_of_unity[i] - x;
    }

    fr_batch_inv(&mut inverses, &inverses_in, qlen);

    for (i, v) in inverses.iter().enumerate().take(qlen) {
        q.coeffs[i] = &q.coeffs[i] * v;
    }

    if m != 0 {
        m -= 1;
        q.coeffs[m] = blsScalar::zero();

        for i in 0..qlen {
            if i == m {
                continue;
            }
            tmp = x - &roots_of_unity[i];
            inverses_in[i] = &tmp * x;
        }

        fr_batch_inv(&mut inverses, &inverses_in, qlen);

        for i in 0..qlen {
            tmp = p.coeffs[i] - y;
            tmp = tmp * roots_of_unity[i];
            tmp = tmp * inverses[i];
            q.coeffs[i] = q.coeffs[i] + tmp;
        }
    }

    g1_lincomb(&s.secret_g1, q.coeffs.as_slice())
}

pub fn verify_kzg_proof(
    polynomial_kzg: &ZkG1Projective,
    z: &blsScalar,
    y: &blsScalar,
    kzg_proof: &ZkG1Projective,
    s: &KZGSettings,
) -> bool {
    check_proof_single(polynomial_kzg, kzg_proof, z, y, s)
        .unwrap_or(false)
}

pub fn compute_aggregate_kzg_proof(blobs: &[Vec<blsScalar>], s: &KZGSettings) -> ZkG1Projective {
    let n = blobs.len();
    if n == 0 {
        return ZkG1Projective::identity();
    }

    #[cfg(feature = "parallel")]
    let (polys, commitments): (Vec<_>, Vec<_>) = blobs
        .par_iter()
        .map(|blob| {
            let poly = poly_from_blob(blob);
            let commitment = poly_to_kzg_commitment(&poly, s);
            (poly, commitment)
        })
        .unzip();

    #[cfg(not(feature = "parallel"))]
    let (polys, commitments): (Vec<_>, Vec<_>) = blobs
        .iter()
        .map(|blob| {
            let poly = poly_from_blob(blob);
            let commitment = poly_to_kzg_commitment(&poly, s);
            (poly, commitment)
        })
        .unzip();

    let (aggregated_poly, _, evaluation_challenge) =
        compute_aggregated_poly_and_commitment(&polys, &commitments, n);
    compute_kzg_proof(&aggregated_poly, &evaluation_challenge, s)
}

pub fn verify_aggregate_kzg_proof(
    blobs: &[Vec<blsScalar>],
    expected_kzg_commitments: &[ZkG1Projective],
    kzg_aggregated_proof: &ZkG1Projective,
    ts: &KZGSettings,
) -> bool {
    #[cfg(feature = "parallel")]
    let polys: Vec<KzgPoly> = blobs.par_iter().map(|blob| poly_from_blob(blob)).collect();
    #[cfg(not(feature = "parallel"))]
    let polys: Vec<KzgPoly> = blobs.iter().map(|blob| poly_from_blob(blob)).collect();

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

fn poly_from_blob(blob: &[blsScalar]) -> KzgPoly {
    let mut p: KzgPoly = KzgPoly::new(blob.len()).unwrap();
    p.coeffs = blob.to_vec();
    p
}

fn compute_aggregated_poly_and_commitment(
    polys: &[KzgPoly],
    kzg_commitments: &[ZkG1Projective],
    n: usize,
) -> (KzgPoly, ZkG1Projective, blsScalar) {
    let mut hash = [0u8; 32];
    hash_to_bytes(&mut hash, polys, kzg_commitments);
    let r = bytes_to_bls_field(&hash);

    let (r_powers, chal_out) = if n == 1 {
        (vec![r], r)
    } else {
        let r_powers = compute_powers(&r, n);
        let chal_out = r_powers[1] * r_powers[n - 1];
        (r_powers, chal_out)
    };

    let poly_out = poly_lincomb(polys, &r_powers);

    let comm_out = g1_lincomb(kzg_commitments, &r_powers);

    (poly_out, comm_out, chal_out)
}

#[cfg(not(feature = "parallel"))]
pub fn poly_lincomb(vectors: &[KzgPoly], scalars: &[blsScalar]) -> KzgPoly {
    let mut res: KzgPoly = KzgPoly::new(FIELD_ELEMENTS_PER_BLOB).unwrap();
    let mut tmp: blsScalar;
    for j in 0..FIELD_ELEMENTS_PER_BLOB {
        res.set_coeff_at(j, &blsScalar::zero());
    }
    let n = vectors.len();
    for i in 0..n {
        for j in 0..FIELD_ELEMENTS_PER_BLOB {
            tmp = scalars[i] * vectors[i].get_coeff_at(j);
            res.set_coeff_at(j, &(res.get_coeff_at(j) + tmp));
        }
    }
    res
}

#[cfg(feature = "parallel")]
pub fn poly_lincomb(vectors: &[KzgPoly], scalars: &[blsScalar]) -> KzgPoly {
    let n = vectors.len();
    KzgPoly {
        coeffs: (0..FIELD_ELEMENTS_PER_BLOB).into_par_iter().map(|j|{
            let mut out = Fr::zero();
            for i in 0..n {
                out += &(scalars[i] * vectors[i].get_coeff_at(j));
            }
            out
        }).collect()
    }
}

fn poly_to_kzg_commitment(p: &KzgPoly, s: &KZGSettings) -> ZkG1Projective {
    g1_lincomb(&s.secret_g1, &p.coeffs)
}

fn hash_to_bytes(out: &mut [u8; 32], polys: &[KzgPoly], comms: &[ZkG1Projective]) {
    let n = polys.len();
    let ni = FIAT_SHAMIR_PROTOCOL_DOMAIN.len() + 8 + 8;
    let np = ni + n * FIELD_ELEMENTS_PER_BLOB * 32;

    let mut bytes = vec![0u8; np + n * 48];

    bytes[0..16].copy_from_slice(&FIAT_SHAMIR_PROTOCOL_DOMAIN);
    bytes[16..24].copy_from_slice(&n.to_le_bytes());
    bytes[24..32].copy_from_slice(&FIELD_ELEMENTS_PER_BLOB.to_le_bytes());

    for (i, poly) in polys.iter().enumerate().take(n) {
        for j in 0..FIELD_ELEMENTS_PER_BLOB {
            let pos = ni + i * BYTES_PER_FIELD_ELEMENT;
            bytes[pos..pos + BYTES_PER_FIELD_ELEMENT]
                .copy_from_slice(&bytes_from_bls_field(&poly.coeffs[j]));
        }
    }

    for (i, comm) in comms.iter().enumerate().take(n) {
        let pos = ni + i * BYTES_PER_FIELD_ELEMENT;
        let data = &bytes_from_g1(comm);
        bytes[pos..pos + data.len()].copy_from_slice(data);
    }

    hash(out, &bytes);
}

fn hash(md: &mut [u8; 32], input: &[u8]) {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize_into(md.try_into().unwrap());
}

pub const FIELD_ELEMENTS_PER_BLOB: usize = 4096;
pub const FIAT_SHAMIR_PROTOCOL_DOMAIN: [u8; 16] = [70, 83, 66, 76, 79, 66, 86, 69, 82, 73, 70, 89, 95, 86, 49, 95]; // "FSBLOBVERIFY_V1_"
pub const BYTES_PER_FIELD_ELEMENT: usize = 32;
