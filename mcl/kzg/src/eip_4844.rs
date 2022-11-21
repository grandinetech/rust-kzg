use crate::data_types::g1::mclBnG1_mulVec;
use crate::data_types::{fr::*, g1::*, g2::*};
use crate::fk20_fft::*;
use crate::kzg10::{Curve, Polynomial};
use crate::kzg_settings::KZGSettings;
use crate::mcl_methods::*;
use kzg::Poly as _;
use sha2::{Digest as _, Sha256};
use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use std::usize;

// [x] bytes_to_bls_field
// [x] vector_lincomb
// [x] g1_lincomb
// [x] blob_to_kzg_commitment
// [x] verify_kzg_proof
// [x] compute_kzg_proof
// [x] evaluate_polynomial_in_evaluation_form

pub fn bytes_to_g1(bytes: &[u8; 48usize]) -> G1 {
    set_eth_serialization(1);
    let mut g1 = G1::default();
    if !G1::deserialize(&mut g1, bytes) {
        panic!("failed to deserialize")
    }
    g1
}
pub fn bytes_to_g2(bytes: &[u8]) -> G2 {
    set_eth_serialization(1);
    let mut g2 = G2::default();
    if !G2::deserialize(&mut g2, bytes) {
        panic!("failed to deserialize")
    }
    g2
}

pub fn bytes_from_g1(g1: &G1) -> [u8; 48usize] {
    set_eth_serialization(1);
    G1::serialize(g1).try_into().unwrap()
}

pub fn load_trusted_setup(filepath: &str) -> KZGSettings {
    let mut file = File::open(filepath).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Unable to read file");

    let mut lines = contents.lines();
    let length = lines.next().unwrap().parse::<usize>().unwrap();
    let n2 = lines.next().unwrap().parse::<usize>().unwrap();

    let mut g2_values: Vec<G2> = Vec::new();

    let mut g1_projectives: Vec<G1> = Vec::new();

    for _ in 0..length {
        let line = lines.next().unwrap();
        assert!(line.len() == 96);
        let bytes: [u8; 48] = (0..line.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&line[i..i + 2], 16).unwrap())
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap();
        g1_projectives.push(bytes_to_g1(&bytes));
    }

    for _ in 0..n2 {
        let line = lines.next().unwrap();
        assert!(line.len() == 192);
        let bytes = (0..line.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&line[i..i + 2], 16).unwrap())
            .collect::<Vec<u8>>();
        g2_values.push(bytes_to_g2(bytes.as_slice()));
    }

    let mut max_scale: usize = 0;
    while (1 << max_scale) < length {
        max_scale += 1;
    }

    //let fs = FFTSettings::new_custom_primitive_roots(max_scale as u8, SCALE_2_ROOT_OF_UNITY_PR5_STRINGS).unwrap();
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

pub fn bytes_to_bls_field(bytes: &[u8; 32usize]) -> Fr {
    Fr::from_scalar(bytes)
}

pub fn bytes_from_bls_field(fr: &Fr) -> [u8; 32usize] {
    Fr::to_scalar(fr)
}

pub fn vector_lincomb(vectors: &[Vec<Fr>], scalars: &[Fr]) -> Vec<Fr> {
    let mut out = vec![Fr::zero(); vectors[0].len()];
    let mut tmp: Fr;
    for (v, s) in vectors.iter().zip(scalars.iter()) {
        for (i, x) in v.iter().enumerate() {
            tmp = x * s;
            out[i] = out[i] + tmp;
        }
    }
    out
}

pub fn g1_lincomb(p: &[G1], coeffs: &[Fr]) -> G1 {
    assert!(p.len() == coeffs.len());

    let mut out = G1::default();
    g1_linear_combination(&mut out, p, coeffs, p.len());

    out
}

pub fn blob_to_kzg_commitment(blob: &[Fr], s: &KZGSettings) -> G1 {
    g1_lincomb(&s.curve.g1_points, blob)
}

pub fn verify_kzg_proof(commitment: &G1, x: &Fr, y: &Fr, proof: &G1, ks: &KZGSettings) -> bool {
    let (mut x_g2, mut s_minus_x) = (G2::default(), G2::default());
    let (mut y_g1, mut commitment_minus_y) = (G1::default(), G1::default());

    G2::mul(&mut x_g2, &G2::gen(), x);
    G2::sub(&mut s_minus_x, &ks.curve.g2_points[1], &x_g2);
    G1::mul(&mut y_g1, &G1::gen(), y);
    G1::sub(&mut commitment_minus_y, commitment, &y_g1);

    Curve::verify_pairing(&commitment_minus_y, &G2::gen(), proof, &s_minus_x)
}

pub fn compute_kzg_proof(p: &Polynomial, x: &Fr, s: &KZGSettings) -> G1 {
    assert!(p.coeffs.len() <= s.curve.g1_points.len());

    let y = evaluate_polynomial_in_evaluation_form(p, x, s);

    let mut tmp: Fr;

    let mut roots_of_unity = s.fft_settings.exp_roots_of_unity.clone();
    reverse_bit_order(&mut roots_of_unity);

    let mut m = 0;

    let mut q = Polynomial::new(p.coeffs.len());
    let plen = p.coeffs.len();
    let qlen = q.coeffs.len();

    let mut inverses_in: Vec<Fr> = vec![Fr::default(); plen];
    let mut inverses: Vec<Fr> = vec![Fr::default(); plen];

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
        q.coeffs[m] = Fr::zero();

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

    g1_lincomb(&s.curve.g1_points, q.coeffs.as_slice())
}

pub fn evaluate_polynomial_in_evaluation_form(p: &Polynomial, x: &Fr, s: &KZGSettings) -> Fr {
    let mut out;
    let mut tmp = Fr::default();
    let mut t: Fr;
    let plen = p.coeffs.len();
    let mut inverses_in = vec![Fr::default(); plen];
    let mut inverses = vec![Fr::default(); plen];
    let mut roots_of_unity = s.fft_settings.exp_roots_of_unity.clone();

    reverse_bit_order(&mut roots_of_unity);

    for i in 0..plen {
        if *x == roots_of_unity[i] {
            return p.coeffs[i];
        }
        Fr::sub(&mut inverses_in[i], x, &roots_of_unity[i]);
    }

    fr_batch_inv(inverses.as_mut_slice(), inverses_in.as_slice(), plen);

    out = Fr::zero();

    for i in 0..plen {
        Fr::mul(&mut tmp, &inverses[i], &roots_of_unity[i]);
        t = tmp;
        Fr::mul(&mut tmp, &t, &p.coeffs[i]);
        t = out;
        Fr::add(&mut out, &t, &tmp);
    }

    let arr: [u64; 4] = [plen.try_into().unwrap(), 0, 0, 0];
    tmp = Fr::from_u64_arr(&arr);
    t = out;
    Fr::div(&mut out, &t, &tmp);
    tmp = x.pow(plen);
    t = tmp;
    Fr::sub(&mut tmp, &t, &Fr::one());
    t = out;
    Fr::mul(&mut out, &t, &tmp);

    out
}

pub fn compute_powers(x: &Fr, n: usize) -> Vec<Fr> {
    let mut out: Vec<Fr> = vec![Fr::default(); n];
    out[0] = Fr::one();
    for i in 1..n {
        out[i] = &out[i - 1] * x;
    }
    out
}

// TODO: add return value
fn fr_batch_inv(out: &mut [Fr], a: &[Fr], len: usize) {
    let mut prod = vec![Fr::default(); len];

    prod[0] = a[0];

    for i in 1..len {
        let t = prod[i - 1];
        Fr::mul(&mut prod[i], &a[i], &t);
    }

    let mut inv = prod[len - 1].inverse();

    for i in (1..len).rev() {
        Fr::mul(&mut out[i], &inv, &prod[i - 1]);
        let t = inv;
        Fr::mul(&mut inv, &a[i], &t);
    }
    out[0] = inv;
}

fn g1_linear_combination(result: &mut G1, g1_points: &[G1], coeffs: &[Fr], n: usize) {
    unsafe { mclBnG1_mulVec(result, g1_points.as_ptr(), coeffs.as_ptr(), n) }
}

pub fn compute_aggregate_kzg_proof(blob: &[Vec<Fr>], s: &KZGSettings) -> G1 {
    let mut commitments: Vec<G1> = vec![G1::default(); blob.len()];
    let mut polys: Vec<Polynomial> = vec![Polynomial::new(FIELD_ELEMENTS_PER_BLOB); blob.len()];
    for i in 0..blob.len() {
        polys[i] = poly_from_blob(&blob[i]);
        commitments[i] = poly_to_kzg_commitment(&polys[i], s);
    }
    let (aggregated_poly, _, evaluation_challenge) =
        compute_aggregated_poly_and_commitment(&polys, &commitments, blob.len());
    compute_kzg_proof(&aggregated_poly, &evaluation_challenge, s)
}

pub fn verify_aggregate_kzg_proof(
    blobs: &[Vec<Fr>],
    expected_kzg_commitments: &[G1],
    kzg_aggregated_proof: &G1,
    ts: &KZGSettings,
) -> bool {
    let mut polys: Vec<Polynomial> = vec![Polynomial::new(FIELD_ELEMENTS_PER_BLOB); blobs.len()];
    for i in 0..blobs.len() {
        polys[i] = poly_from_blob(&blobs[i]);
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

fn poly_from_blob(blob: &[Fr]) -> Polynomial {
    let mut out = Polynomial::new(blob.len());
    out.coeffs[..blob.len()].copy_from_slice(blob);
    out
}

fn poly_to_kzg_commitment(p: &Polynomial, s: &KZGSettings) -> G1 {
    g1_lincomb(&s.curve.g1_points, &p.coeffs)
}

fn compute_aggregated_poly_and_commitment(
    polys: &[Polynomial],
    kzg_commitments: &[G1],
    n: usize,
) -> (Polynomial, G1, Fr) {
    let mut hash = [0u8; 32];
    hash_to_bytes(&mut hash, polys, kzg_commitments); 

    let r = bytes_to_bls_field(&hash);
    let r_powers = compute_powers(&r, n);

    let chal_out = r_powers[1] * r_powers[n - 1];

    let poly_out = poly_lincomb(polys, &r_powers); 

    let comm_out = g1_lincomb(kzg_commitments, &r_powers);

    (poly_out, comm_out, chal_out)
}

fn hash(md: &mut [u8; 32], input: &[u8]) {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize_into(md.try_into().unwrap());
}

fn hash_to_bytes(out: &mut [u8; 32], polys: &[Polynomial], comms: &[G1]) {
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

pub fn poly_lincomb(vectors: &[Polynomial], scalars: &[Fr]) -> Polynomial {
    let mut res: Polynomial = Polynomial::new(FIELD_ELEMENTS_PER_BLOB);
    let mut tmp: Fr;
    for j in 0..FIELD_ELEMENTS_PER_BLOB {
        res.set_coeff_at(j, &Fr::zero());
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

pub const FIELD_ELEMENTS_PER_BLOB: usize = 4096;
pub const FIAT_SHAMIR_PROTOCOL_DOMAIN: [u8; 16] = [70, 83, 66, 76, 79, 66, 86, 69, 82, 73, 70, 89, 95, 86, 49, 95]; // "FSBLOBVERIFY_V1_"
pub const BYTES_PER_FIELD_ELEMENT: usize = 32;
