use std::io::Read;

use std::{convert::TryInto, fs::File, slice};

use crate::consts::{BlstP1, BlstP1Affine, BlstP2, BlstP2Affine, BLST_ERROR};
use crate::finite::{
    blst_p1_compress, blst_p1_from_affine, blst_p1_uncompress, blst_p2_from_affine,
    blst_p2_uncompress, g1_linear_combination, BlstFr,
};

use crate::fftsettings::KzgFFTSettings;
use crate::kzgsettings::KzgKZGSettings;
use crate::poly::KzgPoly;
use crate::utils::reverse_bit_order;

use kzg::{FFTSettings, Fr, KZGSettings, Poly, FFTG1};

pub fn bytes_to_g1(bytes: [u8; 48usize]) -> BlstP1 {
    let mut tmp = BlstP1Affine::default();
    let mut g1 = BlstP1::default();
    unsafe {
        if blst_p1_uncompress(&mut tmp, bytes.as_ptr()) != BLST_ERROR::BLST_SUCCESS {
            panic!("blst_p1_uncompress failed");
        }
        blst_p1_from_affine(&mut g1, &tmp);
    }
    g1
}

pub fn bytes_from_g1(g1: &BlstP1) -> [u8; 48usize] {
    let mut out: [u8; 48usize] = [0; 48];
    unsafe {
        // it say that it is not FFI safe
        blst_p1_compress(out.as_mut_ptr(), g1);
    }
    out
}

pub fn load_trusted_setup(filepath: &str) -> KzgKZGSettings {
    let mut file = File::open(filepath).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    let mut lines = contents.lines();
    let length = lines.next().unwrap().parse::<usize>().unwrap();
    let n2 = lines.next().unwrap().parse::<usize>().unwrap();

    let mut g2_values = Box::new(Vec::new());

    let mut g1_projectives: Vec<BlstP1> = Vec::new();

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
        let mut tmp = BlstP2Affine::default();
        let mut g2 = BlstP2::default();
        unsafe {
            if blst_p2_uncompress(&mut tmp, bytes_array.as_ptr()) != BLST_ERROR::BLST_SUCCESS {
                panic!("blst_p2_uncompress failed");
            }
            blst_p2_from_affine(&mut g2, &tmp);
        }
        g2_values.push(g2);
    }

    let mut max_scale: usize = 0;
    while (1 << max_scale) < length {
        max_scale += 1;
    }

    let boxed = Box::new(KzgFFTSettings::new(max_scale).unwrap());
    let fs = Box::into_raw(boxed);
    let mut g1_values = Box::new(unsafe { (*fs).fft_g1(&g1_projectives, true).unwrap() });

    reverse_bit_order(&mut g1_values);
    assert!(g2_values.len() == g2_values.len());

    KzgKZGSettings {
        length: g1_values.len().try_into().unwrap(),
        secret_g1: unsafe { (*(Box::into_raw(g1_values))).as_mut_ptr() },
        secret_g2: unsafe { (*(Box::into_raw(g2_values))).as_mut_ptr() },
        fs,
    }
    // fs.
}

fn fr_batch_inv(out: &mut [BlstFr], a: &[BlstFr], len: usize) {
    let prod: &mut Vec<BlstFr> = &mut vec![BlstFr::default(); len];
    let mut i: usize = 1;

    prod[0] = a[0];

    while i < len {
        prod[i] = a[i].mul(&prod[i - 1]);
        i += 1;
    }

    let inv: &mut BlstFr = &mut prod[len - 1].eucl_inverse();

    i = len - 1;
    while i > 0 {
        out[i] = prod[i - 1].mul(inv);
        *inv = a[i].mul(inv);
        i -= 1;
    }
    out[0] = *inv;
}

pub fn bytes_to_bls_field(bytes: &[u8; 32usize]) -> BlstFr {
    let my_u64_vec = unsafe { (bytes[..32].align_to::<u64>().1).to_vec() };
    let arr: [u64; 4] = match my_u64_vec.try_into() {
        Ok(arr) => arr,
        Err(_) => panic!(),
    };
    BlstFr::from_u64_arr(&arr)
}

pub fn vector_lincomb(vectors: &[Vec<BlstFr>], scalars: &[BlstFr]) -> Vec<BlstFr> {
    let mut tmp: BlstFr;
    let mut out: Vec<BlstFr> = vec![BlstFr::zero(); vectors[0].len()];
    for (v, s) in vectors.iter().zip(scalars.iter()) {
        for (i, x) in v.iter().enumerate() {
            tmp = x.mul(s);
            out[i] = out[i].add(&tmp);
        }
    }
    out
}

pub fn bytes_from_bls_field(fr: &BlstFr) -> [u8; 32usize] {
    // probably this and bytes_to_bls_field can be rewritten in blst functions
    let v = &fr.to_u64_arr();
    // investigate if being little endian changes something
    // order of bytes might need to be reversed
    let my_u8_vec_bis: Vec<u8> = unsafe { (v[..4].align_to::<u8>().1).to_vec() };
    my_u8_vec_bis.try_into().unwrap()
}

pub fn g1_lincomb(points: &[BlstP1], scalars: &[BlstFr]) -> BlstP1 {
    // yra linear_combination_g1, kuris - safe
    assert!(points.len() == scalars.len());
    let mut out = BlstP1::default();
    unsafe {
        g1_linear_combination(
            &mut out,
            points.as_ptr(),
            scalars.as_ptr(),
            points.len().try_into().unwrap(),
        );
    }
    out
}

pub fn blob_to_kzg_commitment(blob: &[BlstFr], s: &KzgKZGSettings) -> BlstP1 {
    g1_lincomb(
        unsafe { slice::from_raw_parts(s.secret_g1, s.length.try_into().unwrap()) },
        blob,
    )
}

pub fn verify_kzg_proof(
    polynomial_kzg: &BlstP1,
    z: &BlstFr,
    y: &BlstFr,
    kzg_proof: &BlstP1,
    s: &KzgKZGSettings,
) -> bool {
    s.check_proof_single(polynomial_kzg, kzg_proof, z, y)
        .unwrap_or(false)
}

pub fn compute_kzg_proof(p: &mut KzgPoly, x: &BlstFr, s: &KzgKZGSettings) -> BlstP1 {
    // Here parts of KzgSettings are converted to Vecs and Slices
    let secret_g1 =
        unsafe { slice::from_raw_parts(s.secret_g1, s.length.try_into().unwrap()).to_vec() };

    assert!(p.len() <= secret_g1.len());

    let y: BlstFr = evaluate_polynomial_in_evaluation_form(p, x, s);

    let mut tmp: BlstFr;
    let mut roots_of_unity: Vec<BlstFr> = unsafe {
        slice::from_raw_parts(
            (*s.fs).expanded_roots_of_unity,
            s.length.try_into().unwrap(),
        )
        .to_vec()
    };

    reverse_bit_order(&mut roots_of_unity);
    let mut i: usize = 0;
    let mut m: usize = 0;

    let mut q: KzgPoly = KzgPoly::new(p.len()).unwrap();

    let mut inverses_in: Vec<BlstFr> = vec![BlstFr::default(); p.len()];
    let mut inverses: Vec<BlstFr> = vec![BlstFr::default(); p.len()];

    while i < q.len() {
        if x.equals(&roots_of_unity[i]) {
            m = i + 1;
            continue;
        }

        // (p_i - y) / (ω_i - x)
        q.set_coeff_at(i, &(p.get_coeff_at(i).sub(&y)));
        inverses_in[i] = roots_of_unity[i].sub(x);
        i += 1;
    }

    fr_batch_inv(&mut inverses, &inverses_in, q.len());

    i = 0;
    while i < q.len() {
        q.set_coeff_at(i, &q.get_coeff_at(i).mul(&inverses[i]));
        i += 1;
    }

    if m > 0 {
        // ω_m == x
        q.set_coeff_at(m, &BlstFr::zero());

        m -= 1;
        i = 0;
        while i < q.len() {
            if i == m {
                continue;
            }
            // (p_i - y) * ω_i / (x * (x - ω_i))
            tmp = x.sub(&roots_of_unity[i]);
            inverses_in[i] = tmp.mul(x);
            i += 1;
        }
        fr_batch_inv(&mut inverses, &inverses_in, q.len());
        i = 0;
        while i < q.len() {
            tmp = (p.get_coeff_at(i)).sub(&y);
            tmp = tmp.mul(&roots_of_unity[i]);
            tmp = tmp.mul(&inverses[i]);
            q.set_coeff_at(m, &(q.get_coeff_at(m)).add(&tmp));
            i += 1;
        }
    }

    g1_lincomb(&secret_g1, q.get_coeffs())
}

pub fn evaluate_polynomial_in_evaluation_form(
    p: &KzgPoly,
    x: &BlstFr,
    s: &KzgKZGSettings,
) -> BlstFr {
    let mut tmp: BlstFr;

    let mut inverses_in: Vec<BlstFr> = vec![BlstFr::default(); p.len()];
    let mut inverses: Vec<BlstFr> = vec![BlstFr::default(); p.len()];
    let mut i: usize = 0;
    let mut roots_of_unity = unsafe {
        slice::from_raw_parts((*s.fs).expanded_roots_of_unity, (*s.fs).max_width).to_vec()
    };

    reverse_bit_order(&mut roots_of_unity);

    while i < p.len() {
        if x.equals(&roots_of_unity[i]) {
            return p.get_coeff_at(i);
        }

        inverses_in[i] = x.sub(&roots_of_unity[i]);
        i += 1;
    }
    fr_batch_inv(&mut inverses, &inverses_in, p.len());

    let mut out = BlstFr::zero();
    i = 0;
    while i < p.len() {
        tmp = inverses[i].mul(&roots_of_unity[i]);
        tmp = tmp.mul(&p.get_coeff_at(i));
        out = out.add(&tmp);
        i += 1;
    }
    tmp = BlstFr::from_u64(p.len().try_into().unwrap());
    out = out.div(&tmp).unwrap();
    tmp = x.pow(p.len());
    tmp = tmp.sub(&BlstFr::one());
    out = out.mul(&tmp);
    out
}

pub fn compute_powers(base: &BlstFr, num_powers: usize) -> Vec<BlstFr> {
    let mut powers: Vec<BlstFr> = vec![BlstFr::default(); num_powers];
    powers[0] = BlstFr::one();
    for i in 1..num_powers {
        powers[i] = powers[i - 1].mul(base);
    }
    powers
}
