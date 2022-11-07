use crate::data_types::g1::mclBnG1_mulVec;
use crate::fk20_fft::*;
use crate::data_types::{fr::*, g1::*, g2::*};
use crate::kzg10::{Curve, Polynomial};
use crate::kzg_settings::KZGSettings;
use std::convert::TryInto;
use std::usize;
use crate::mcl_methods::*;
use std::fs::File;
use std::io::Read;

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
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

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
        curve: Curve { g1_gen: G1::gen(), g2_gen: G2::gen(), g1_points: g1_values, g2_points: g2_values }
    }
}

pub fn reverse_bit_order<T>(values: &mut [T]) where T: Clone {
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

pub fn verify_kzg_proof(
    commitment: &G1,
    x: &Fr,
    y: &Fr,
    proof: &G1,
    ks: &KZGSettings,
) -> bool {
    let (mut x_g2, mut s_minus_x) = (G2::default(), G2::default());
    let (mut y_g1, mut commitment_minus_y) = (G1::default(), G1::default());

    G2::mul(&mut x_g2, &G2::gen(), x);
    G2::sub(&mut s_minus_x, &ks.curve.g2_points[1], &x_g2);
    G1::mul(&mut y_g1, &G1::gen(), y);
    G1::sub(&mut commitment_minus_y, commitment, &y_g1);

    Curve::verify_pairing(&commitment_minus_y, &G2::gen(), proof, &s_minus_x)
}

pub fn compute_kzg_proof(p: &mut Polynomial, x: &Fr, s: &KZGSettings) -> G1 {
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
        p.coeffs[m] = Fr::zero();

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


// TODO: add return value
pub fn evaluate_polynomial_in_evaluation_form(
    p: &Polynomial,
    x: &Fr,
    s: &KZGSettings,
) -> Fr {
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


fn compute_aggregate_kzg_proof(
    blob: &[Fr],
    n: usize,
    s: &KZGSettings
    ) -> G1
{

    let mut KZGCommitments: Vec<G1> = Vec::new();
    let mut polys = Polynomial::new(n);


    for i in 0..n {
        polys.coeffs.push(    (&poly_from_blob(blob)).coeffs[i]   );
        KZGCommitments.push(  poly_to_kzg_commitment(polys, s)  );

    }


    let mut aggregated_poly: Polynomial;
    let mut aggregated_poly_commitment: G1;
    let mut evaluation_challenge: Fr;
    let mut out = compute_aggregated_poly_and_commitment( 
        evaluation_challenge, 
        &[polys], 
        &KZGCommitments, 
        n);


    return compute_kzg_proof(&mut aggregated_poly, &evaluation_challenge, s);
}



//- verify_aggregate_kzg_proof()
/* 
C_KZG_RET verify_aggregate_kzg_proof(bool *out,
    const Blob blobs[],
    const KZGCommitment expected_kzg_commitments[],
    size_t n,
    const KZGProof *kzg_aggregated_proof,
    const KZGSettings *s) {
  Polynomial* polys = calloc(n, sizeof(Polynomial));
  if (polys == NULL) return C_KZG_MALLOC;
  for (size_t i = 0; i < n; i++)
    poly_from_blob(polys[i], blobs[i]);

  Polynomial aggregated_poly;
  KZGCommitment aggregated_poly_commitment;
  BLSFieldElement evaluation_challenge;
  C_KZG_RET ret;
  ret = compute_aggregated_poly_and_commitment(aggregated_poly, &aggregated_poly_commitment, &evaluation_challenge, polys, expected_kzg_commitments, n);
  free(polys);
  if (ret != C_KZG_OK) return ret;

  BLSFieldElement y;
  TRY(evaluate_polynomial_in_evaluation_form(&y, aggregated_poly, &evaluation_challenge, s));

  return verify_kzg_proof(out, &aggregated_poly_commitment, &evaluation_challenge, &y, kzg_aggregated_proof, s);
}
*/

fn poly_from_blob(blob: &[Fr]) -> Polynomial {
    let mut out = Polynomial::new(0);
    for i in 0..420 { //Where to get the upper bound??
        out.coeffs.push(
            bytes_to_bls_field( 
                &Fr::to_scalar(&blob[i])
            )
        );
    }
    out
}



fn poly_to_kzg_commitment(p: Polynomial, s: &KZGSettings) -> G1{
    let mut out = G1::default();
    out = g1_lincomb(&s.curve.g1_points, &p.coeffs);
    out
}


fn compute_aggregated_poly_and_commitment( 
    chal_out: Fr,
    polys: &[Polynomial],
    kzg_commitments: &[G1],
    n: usize)  -> (Polynomial, G1)
{
    let mut r_powers: &[Fr];
    let mut hash: &[u8; 32];

    let mut polynomial_out: Polynomial;
    let mut KZGCommitment_out: G1;

    hash_to_bytes(hash, polys, kzg_commitments, n); //Implementation missing

    r_powers[1] = bytes_to_bls_field(hash);

    compute_powers(&r_powers[0], n);
    chal_out = &r_powers[1] * &r_powers[n - 1];

    poly_lincomb(polynomial_out, polys, r_powers, n); //Implementation missing, probably move polynomial_out out from argument list

    KZGCommitment_out = g1_lincomb(kzg_commitments, r_powers);


    (polynomial_out, KZGCommitment_out) 
}

//Implement function
fn hash_to_bytes(
    a: u8, 
    polys: &[Polynomial],
    kzg_commitments: &[G1],
    n: usize) { 
 
}

//Implement function
fn poly_lincomb(
    out: Polynomial, 
    vectors:  &[Polynomial], 
    scalars: &[Fr], 
    n: usize) {
    
  }