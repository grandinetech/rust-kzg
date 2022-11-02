use std::io::Read;

use std::{convert::TryInto, fs::File, slice};

use crate::consts::{BlstP1, BlstP1Affine, BlstP2, BLST_ERROR, BlstP2Affine};
use crate::finite::{blst_p1_from_affine, blst_p1_uncompress, BlstFr, blst_p2_from_affine, blst_p2_uncompress};

use crate::fftsettings::{KzgFFTSettings};
use crate::kzgsettings::KzgKZGSettings;
use crate::poly::KzgPoly;
use crate::utils::reverse_bit_order;

use kzg::{Fr, FFTSettings, FFTG1, Poly};

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

    let  boxed = Box::new(KzgFFTSettings::new(max_scale).unwrap());
    let fs = Box::into_raw(boxed);
    let mut g1_values = Box::new(unsafe{
        (*fs).fft_g1(&g1_projectives, true).unwrap()
    });

    reverse_bit_order(&mut g1_values);

    KzgKZGSettings {
        length: g1_values.len().try_into().unwrap(),
        secret_g1: unsafe {(*(Box::into_raw(g1_values))).as_mut_ptr() },
        secret_g2: unsafe {(*(Box::into_raw(g2_values))).as_mut_ptr() },
        fs: fs,
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

pub fn bytes_from_bls_field(fr: &BlstFr) -> [u8; 32usize] {
    // probably this and bytes_to_bls_field can be rewritten in blst functions
    let v = &fr.to_u64_arr();
    // investigate if being little endian changes something
    // order of bytes might need to be reversed
    let my_u8_vec_bis: Vec<u8> = unsafe { (v[..4].align_to::<u8>().1).to_vec() };
    my_u8_vec_bis.try_into().unwrap()
}

pub fn bytes_to_bls_field(bytes: &[u8; 32usize]) -> BlstFr {
    let my_u64_vec = unsafe { (bytes[..32].align_to::<u64>().1).to_vec() };
    let arr: [u64; 4] = match my_u64_vec.try_into() {
        Ok(arr) => arr,
        Err(_) => panic!(),
    };
    BlstFr::from_u64_arr(&arr)
}

pub fn evaluate_polynomial_in_evaluation_form(p: &KzgPoly, x: &BlstFr, s: &KzgKZGSettings) -> BlstFr {
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
