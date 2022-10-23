use std::convert::TryInto;
use std::fs::File;
use std::io::Read;

use blst::{
    blst_p1, blst_p1_affine, blst_p1_compress, blst_p1_from_affine, blst_p1_uncompress, blst_p2,
    blst_p2_affine, blst_p2_from_affine, blst_p2_uncompress, BLST_ERROR,
};
use kzg::{FFTSettings, Fr, KZGSettings, Poly, FFTG1, G1};

use crate::types::fft_settings::FsFFTSettings;
use crate::types::fr::FsFr;
use crate::types::g1::FsG1;

use crate::kzg_proofs::g1_linear_combination;
use crate::types::g2::FsG2;
use crate::types::kzg_settings::FsKZGSettings;
use crate::types::poly::FsPoly;
use crate::utils::reverse_bit_order;

pub fn bytes_to_g1(bytes: [u8; 48usize]) -> FsG1 {
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
        let bytes = (0..line.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&line[i..i + 2], 16).unwrap())
            .collect::<Vec<u8>>();
        let mut bytes_array: [u8; 48] = [0; 48];
        bytes_array.copy_from_slice(&bytes);
        g1_projectives.push(bytes_to_g1(bytes_array));
    }

    for _ in 0..n2 {
        let line = lines.next().unwrap();
        let bytes = (0..line.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&line[i..i + 2], 16).unwrap())
            .collect::<Vec<u8>>();
        let mut bytes_array: [u8; 96] = [0; 96];
        bytes_array.copy_from_slice(&bytes);
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

// look into &Vec<FsFr> vs &[FsFr] and whether to use &[Vec<FsFr>] or &[&[FsFr]]
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

pub fn compute_kzg_proof(p: &mut FsPoly, x: &FsFr, s: &FsKZGSettings) -> FsG1 {
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
    powers[0] = FsFr::one();
    for i in 1..num_powers {
        powers[i] = powers[i - 1].mul(base);
    }
    powers
}
