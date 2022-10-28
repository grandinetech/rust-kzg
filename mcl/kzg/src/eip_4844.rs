use crate::data_types::g1::mclBnG1_mulVec;
use crate::fk20_fft::*;
use crate::data_types::{fr::*, g1::*, g2::*};
use crate::kzg10::{Curve, Polynomial};
use crate::kzg_settings::KZGSettings;
use std::convert::TryInto;
use crate::mcl_methods::*;
use std::fs::File;
use std::io::Read;

// [x] bytes_to_bls_field
// [x] vector_lincomb
// [x] g1_lincomb
// [x] blob_to_kzg_commitment
// [x] verify_kzg_proof
// [ ] compute_kzg_proof
// [x] evaluate_polynomial_in_evaluation_form

pub fn bytes_to_g1(bytes: &[u8; 48usize]) -> G1 {
    let mut g1 = G1::default();
    if !G1::deserialize(&mut g1, bytes) {
        panic!("failed to deserialize")
    }
    g1
}
pub fn bytes_to_g2(bytes: &[u8]) -> G2 {
    let mut g2 = G2::default();
    if !G2::deserialize(&mut g2, bytes) {
        panic!("failed to deserialize")
    }
    g2
}

pub fn load_trusted_setup(filepath: &str) -> KZGSettings {
    set_eth_serialization(1);

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

pub fn vector_lincomb(out: &mut [Fr], vectors: &[Fr], scalars: &[Fr], n: usize, m: usize) {
    let mut tmp: Fr = Fr::default();
    for o in out.iter_mut() {
        *o = Fr::zero();
    }
    for i in 0..n {
        for j in 0..m {
            Fr::mul(&mut tmp, &scalars[i], &vectors[i * m + j]);
            let t: Fr = out[j];
            Fr::add(&mut out[j], &t, &tmp);
        }
    }
}

pub fn g1_lincomb(out: &mut G1, p: &[G1], coeffs: &[Fr], len: usize) {
    g1_linear_combination(out, p, coeffs, len);
}

pub fn blob_to_kzg_commitment(out: &mut G1, blob: Vec<Fr>, s: &KZGSettings) {
    g1_lincomb(out, &s.curve.g1_points, &blob, s.curve.g1_points.len());
}

pub fn verify_kzg_proof(
    out: &mut bool,
    commitment: &G1,
    x: &Fr,
    y: &Fr,
    proof: &G1,
    ks: &KZGSettings,
) {
    let (mut x_g2, mut s_minus_x) = (G2::default(), G2::default());
    let (mut y_g1, mut commitment_minus_y) = (G1::default(), G1::default());

    G2::mul(&mut x_g2, &G2::gen(), x);
    G2::sub(&mut s_minus_x, &ks.curve.g2_points[1], &x_g2);
    G1::mul(&mut y_g1, &G1::gen(), y);
    G1::sub(&mut commitment_minus_y, commitment, &y_g1);

    *out = Curve::verify_pairing(&commitment_minus_y, &G2::gen(), proof, &s_minus_x);
}

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
