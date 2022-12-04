use std::{convert::TryInto, fs::File};

use crate::consts::{BlstP1, KzgRet};
use crate::finite::BlstFr;

use crate::kzgsettings4844::KzgKZGSettings4844;

// use crate::utils::reverse_bit_order;

use kzg::{Fr, KZGSettings};
use libc::{fdopen, FILE};
use std::ffi::CStr;
use std::os::unix::io::IntoRawFd;

extern "C" {
    fn bytes_to_bls_field(out: *mut BlstFr, bytes: *const u8);
    fn bytes_to_g1(out: *mut BlstP1, bytes: *const u8);
    fn bytes_from_g1(out: *mut u8, g1: *const BlstP1);
    fn load_trusted_setup(out: *mut KzgKZGSettings4844, inp: *mut FILE) -> KzgRet;
    fn verify_aggregate_kzg_proof(
        out: *mut bool,
        blobs: *const u8,
        expected_kzg_commitments: *const BlstP1,
        n: usize,
        kzg_aggregated_proof: *const BlstP1,
        s: *const KzgKZGSettings4844,
    ) -> KzgRet;
    fn blob_to_kzg_commitment(out: *mut BlstP1, blob: *const u8, s: *const KzgKZGSettings4844);
    fn compute_aggregate_kzg_proof(
        out: *mut BlstP1,
        blobs: *const u8,
        n: usize,
        s: *const KzgKZGSettings4844,
    ) -> KzgRet;
}

pub fn bytes_to_g1_rust(bytes: [u8; 48usize]) -> BlstP1 {
    unsafe {
        let g1 = &mut BlstP1::default();
        bytes_to_g1(g1, bytes.as_ptr());
        *g1
    }
}

pub fn bytes_from_g1_rust(g1: &BlstP1) -> [u8; 48usize] {
    let mut out: [u8; 48usize] = [0; 48];
    unsafe {
        bytes_from_g1(out.as_mut_ptr(), g1);
    }
    out
}

pub fn load_trusted_setup_rust(filepath: &str) -> KzgKZGSettings4844 {
    // // https://www.reddit.com/r/rust/comments/8sfjp6/converting_between_file_and_stdfsfile/
    let boxed: Box<KzgKZGSettings4844> = Box::new(KzgKZGSettings4844::default());
    let v = Box::<KzgKZGSettings4844>::into_raw(boxed);
    let res = unsafe {
        let rust_file = File::open(filepath).unwrap();
        let c_file = fdopen(
            rust_file.into_raw_fd(),
            CStr::from_bytes_with_nul_unchecked(b"r\0").as_ptr(),
        );

        load_trusted_setup(v, c_file);

        *Box::<KzgKZGSettings4844>::from_raw(v)
    };
    res
}

pub fn bound_bytes_to_bls_field(bytes: &[u8; 32usize]) -> BlstFr {
    let mut out = BlstFr::default();
    unsafe {
        bytes_to_bls_field(&mut out, bytes.as_ptr());
    }
    out
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

pub fn blob_to_kzg_commitment_rust(blob: &[BlstFr], s: &KzgKZGSettings4844) -> BlstP1 {
    let mut out = BlstP1::default();

    let blob_arr: [u8; 131072usize] = blob
        .iter()
        .flat_map(bytes_from_bls_field)
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap();

    unsafe {
        blob_to_kzg_commitment(&mut out, blob_arr.as_ptr(), s);
    }

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

pub fn compute_aggregate_kzg_proof_rust(blobs: &[Vec<BlstFr>], ts: &KzgKZGSettings4844) -> BlstP1 {
    let mut out = BlstP1::default();
    let blob_arr: Vec<u8> = blobs
        .concat()
        .iter()
        .flat_map(bytes_from_bls_field)
        .collect::<Vec<u8>>();

    unsafe {
        let ret = compute_aggregate_kzg_proof(&mut out, blob_arr.as_ptr(), blobs.len(), ts);
        assert!(ret == KzgRet::KzgOk);
    }
    out
}

pub fn verify_aggregate_kzg_proof_rust(
    blobs: &[Vec<BlstFr>],
    expected_kzg_commitments: &[BlstP1],
    kzg_aggregated_proof: &BlstP1,
    ts: &KzgKZGSettings4844,
) -> bool {
    let mut out = false;
    let blob_arr: Vec<u8> = blobs
        .concat()
        .iter()
        .flat_map(bytes_from_bls_field)
        .collect::<Vec<u8>>();

    unsafe {
        verify_aggregate_kzg_proof(
            &mut out,
            blob_arr.as_ptr(),
            expected_kzg_commitments.as_ptr(),
            blobs.len(),
            kzg_aggregated_proof as *const BlstP1,
            ts,
        );
    }
    out
}
