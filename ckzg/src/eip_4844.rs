use std::{convert::TryInto, fs::File};

use crate::consts::{BlstP1, KzgRet};
use crate::finite::BlstFr;

use crate::kzgsettings4844::KzgKZGSettings4844;

// use crate::utils::reverse_bit_order;

use kzg::eip_4844::{Blob, KZGCommitment, KZGProof};
use kzg::{Fr, KZGSettings};
use libc::{fdopen, FILE};
use std::ffi::CStr;
use std::os::unix::io::IntoRawFd;

extern "C" {
    fn bytes_to_g1(out: *mut BlstP1, bytes: *const u8);
    fn bytes_from_g1(out: *mut u8, g1: *const BlstP1);
    fn load_trusted_setup_file(out: *mut KzgKZGSettings4844, inp: *mut FILE) -> KzgRet;
    fn verify_aggregate_kzg_proof(
        out: *mut bool,
        blobs: *const Blob,
        expected_kzg_commitments: *const KZGCommitment,
        n: usize,
        kzg_aggregated_proof: *const KZGProof,
        s: *const KzgKZGSettings4844,
    ) -> KzgRet;
    fn blob_to_kzg_commitment(
        out: *mut KZGCommitment,
        blob: *const Blob,
        s: *const KzgKZGSettings4844,
    );
    fn compute_aggregate_kzg_proof(
        out: *mut KZGProof,
        blobs: *const Blob,
        n: usize,
        s: *const KzgKZGSettings4844,
    ) -> KzgRet;
}

pub fn load_trusted_setup_rust(filepath: &str) -> KzgKZGSettings4844 {
    // // https://www.reddit.com/r/rust/comments/8sfjp6/converting_between_file_and_stdfsfile/
    let mut v = KzgKZGSettings4844::default();
    unsafe {
        let rust_file = File::open(filepath).unwrap();
        let c_file = fdopen(
            rust_file.into_raw_fd(),
            CStr::from_bytes_with_nul_unchecked(b"r\0").as_ptr(),
        );
        load_trusted_setup_file(&mut v, c_file);
        v
    }
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

    let blob_arr: Blob = Blob {
        bytes: blob
            .iter()
            .flat_map(bytes_from_bls_field)
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap(),
    };

    unsafe {
        let mut kzg_commitment: KZGCommitment = KZGCommitment { bytes: [0; 48] };
        blob_to_kzg_commitment(&mut kzg_commitment, &blob_arr, s);
        bytes_to_g1(&mut out, kzg_commitment.bytes.as_ptr());
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
    let blob_arr: Vec<Blob> = blobs
        .iter()
        .map(|blob| Blob {
            bytes: blob
                .iter()
                .flat_map(bytes_from_bls_field)
                .collect::<Vec<u8>>()
                .try_into()
                .unwrap(),
        })
        .collect::<Vec<Blob>>();

    unsafe {
        let mut kzg_proof: KZGProof = KZGProof { bytes: [0; 48] };
        let ret = compute_aggregate_kzg_proof(&mut kzg_proof, blob_arr.as_ptr(), blobs.len(), ts);
        assert!(ret == KzgRet::KzgOk);
        bytes_to_g1(&mut out, kzg_proof.bytes.as_ptr());
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
    let blob_arr: Vec<Blob> = blobs
        .iter()
        .map(|blob| Blob {
            bytes: blob
                .iter()
                .flat_map(bytes_from_bls_field)
                .collect::<Vec<u8>>()
                .try_into()
                .unwrap(),
        })
        .collect::<Vec<Blob>>();

    let commitments = expected_kzg_commitments
        .iter()
        .map(|c| {
            let mut out: [u8; 48usize] = [0; 48usize];
            unsafe {
                bytes_from_g1(out.as_mut_ptr(), c);
            }
            KZGCommitment { bytes: out }
        })
        .collect::<Vec<KZGCommitment>>();

    let proof = {
        let mut out: [u8; 48usize] = [0; 48usize];
        unsafe {
            bytes_from_g1(out.as_mut_ptr(), kzg_aggregated_proof);
        }
        KZGProof { bytes: out }
    };

    unsafe {
        verify_aggregate_kzg_proof(
            &mut out,
            blob_arr.as_ptr(),
            commitments.as_ptr(),
            blobs.len(),
            &proof,
            ts,
        );
    }
    out
}
