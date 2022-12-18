use std::convert::TryInto;
use std::ffi::{c_char, CStr};
use std::fs::File;
use std::io::Read;

use blst::{
    blst_p1, blst_p1_affine, blst_p1_compress, blst_p1_from_affine, blst_p1_uncompress, blst_p2,
    blst_p2_affine, blst_p2_from_affine, blst_p2_uncompress, BLST_ERROR, blst_fr,
};
use kzg::{FFTSettings, Fr, KZGSettings, Poly, FFTG1, G1};

#[cfg(feature = "parallel")]
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
#[cfg(feature = "parallel")]
use rayon::iter::IntoParallelIterator;

use sha2::{Digest, Sha256};

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

pub fn bytes_to_g1_rust(bytes: &[u8; 48usize]) -> FsG1 {
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

pub fn bytes_from_g1_rust(g1: &FsG1) -> [u8; 48usize] {
    let mut out: [u8; 48usize] = [0; 48];
    unsafe {
        blst_p1_compress(out.as_mut_ptr(), &g1.0);
    }
    out
}

pub fn load_trusted_setup_rust(filepath: &str) -> FsKZGSettings {
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
        g1_projectives.push(bytes_to_g1_rust(&bytes_array));
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

pub fn bytes_to_bls_field_rust(bytes: &[u8; 32usize]) -> FsFr {
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

pub fn blob_to_kzg_commitment_rust(blob: &[FsFr], s: &FsKZGSettings) -> FsG1 {
    g1_lincomb(&s.secret_g1, blob)
}

pub fn verify_kzg_proof_rust(
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
            bytes[ni + BYTES_PER_FIELD_ELEMENT * (i * FIELD_ELEMENTS_PER_BLOB + j) as usize
                ..ni + BYTES_PER_FIELD_ELEMENT * (i * FIELD_ELEMENTS_PER_BLOB + j) + 32]
                .copy_from_slice(&v);
        }
    }

    for i in 0..n {
        let v = bytes_from_g1_rust(&comms[i]);
        for k in 0..48 {
            bytes[np + i * 48 + k] = v[k];
        }
    }

    hash(&bytes)
}

pub fn poly_lincomb(vectors: &[FsPoly], scalars: &[FsFr], n: usize) -> FsPoly {
    #[cfg(not(feature = "parallel"))]
    {
        let mut out: FsPoly = FsPoly::new(FIELD_ELEMENTS_PER_BLOB).unwrap();
        out.coeffs = vec![FsFr::zero(); FIELD_ELEMENTS_PER_BLOB];
        for i in 0..n {
            for j in 0..FIELD_ELEMENTS_PER_BLOB {
                let tmp = scalars[i].mul(&vectors[i].get_coeff_at(j));
                out.set_coeff_at(j, &out.get_coeff_at(j).add(&tmp));
            }
        }
        out    
    }
    #[cfg(feature = "parallel")]
    {
        let mut out: FsPoly = FsPoly::new(FIELD_ELEMENTS_PER_BLOB).unwrap();
        
        out.coeffs = (0..FIELD_ELEMENTS_PER_BLOB).into_par_iter().map(|j|{
            let mut tmp = FsFr::zero();
            for i in 0..n{
                tmp = tmp.add(&scalars[i].mul(&vectors[i].get_coeff_at(j)));
            }
            tmp
        }).collect();    
        out    

    }
}

pub fn compute_aggregated_poly_and_commitment(
    polys: &[FsPoly],
    kzg_commitments: &[FsG1],
    n: usize,
) -> (FsPoly, FsG1, FsFr) {
    let hash = hash_to_bytes(polys, kzg_commitments, n);
    let r = bytes_to_bls_field_rust(&hash);

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

fn poly_from_blob(blob: &[FsFr]) -> FsPoly {
    let mut p: FsPoly = FsPoly::new(FIELD_ELEMENTS_PER_BLOB).unwrap();
    p.coeffs = blob.to_vec();
    p
}

fn poly_to_kzg_commitment(p: &FsPoly, s: &FsKZGSettings) -> FsG1 {
    g1_lincomb(&s.secret_g1, &p.coeffs)
}

pub fn compute_aggregate_kzg_proof_rust(blobs: &[Vec<FsFr>], ts: &FsKZGSettings) -> FsG1 {
    let n = blobs.len();
    if n == 0 {
        return FsG1::identity();
    }

    #[cfg(feature = "parallel")]
    let (polys, commitments): (Vec<_>, Vec<_>) = blobs
        .par_iter()
        .map(|blob| {
            let poly = poly_from_blob(blob);
            let commitment = poly_to_kzg_commitment(&poly, ts);
            (poly, commitment)
        })
        .unzip();

    #[cfg(not(feature = "parallel"))]
    let (polys, commitments): (Vec<_>, Vec<_>) = blobs
        .iter()
        .map(|blob| {
            let poly = poly_from_blob(blob);
            let commitment = poly_to_kzg_commitment(&poly, ts);
            (poly, commitment)
        })
        .unzip();

    let (aggregated_poly, _, evaluation_challenge) =
        compute_aggregated_poly_and_commitment(&polys, &commitments, n);
    compute_kzg_proof(&aggregated_poly, &evaluation_challenge, ts)
}

pub fn verify_aggregate_kzg_proof_rust(
    blobs: &[Vec<FsFr>],
    expected_kzg_commitments: &[FsG1],
    kzg_aggregated_proof: &FsG1,
    ts: &FsKZGSettings,
) -> bool {
    #[cfg(feature = "parallel")]
    let polys: Vec<FsPoly> = blobs.par_iter().map(|blob| poly_from_blob(blob)).collect();
    #[cfg(not(feature = "parallel"))]
    let polys: Vec<FsPoly> = blobs.iter().map(|blob| poly_from_blob(blob)).collect();

    let (aggregated_poly, aggregated_poly_commitment, evaluation_challenge) =
        compute_aggregated_poly_and_commitment(&polys, expected_kzg_commitments, blobs.len());
    let y = evaluate_polynomial_in_evaluation_form(&aggregated_poly, &evaluation_challenge, ts);
    verify_kzg_proof_rust(
        &aggregated_poly_commitment,
        &evaluation_challenge,
        &y,
        kzg_aggregated_proof,
        ts,
    )
}

#[no_mangle]
/// # Safety
///
/// This function should not be called before the horsemen are ready.
pub unsafe extern "C" fn bytes_to_g1(out: *mut blst_p1, bytes: *const u8) {
    let mut tmp = [0u8; 48];
    tmp.copy_from_slice(std::slice::from_raw_parts(bytes, 48));
    *out = bytes_to_g1_rust(&tmp).0;
}

#[no_mangle]
/// # Safety
///
/// This function should not be called before the horsemen are ready.
pub unsafe extern "C" fn bytes_from_g1(out: *mut u8, g1: *const blst_p1) {
    let mut tmp = bytes_from_g1_rust(&FsG1(*g1));
    *out = *tmp.as_mut_ptr();
}

#[no_mangle]
/// # Safety
///
/// This function should not be called before the horsemen are ready.
pub unsafe extern "C" fn bytes_to_bls_field(out: *mut blst_fr, bytes: *const u8) {
    let mut tmp = [0u8; 32];
    tmp.copy_from_slice(std::slice::from_raw_parts(bytes, 32) );
    *out = bytes_to_bls_field_rust(&tmp).0;
}

pub struct CFsFFTSettings{
    pub max_width: u64,
    pub expanded_roots_of_unity: *mut FsFr,
    pub reverse_roots_of_unity: *mut FsFr,
    pub roots_of_unity: *mut FsFr,

}

pub struct CFsKzgSettings{
    pub fs: CFsFFTSettings,
    pub g1_values: *mut FsG1, // G1
    pub g2_values: *mut FsG2, // G2

}

fn fft_settings_to_rust(c_settings: &CFsFFTSettings) -> FsFFTSettings{
    FsFFTSettings{
        max_width: c_settings.max_width as usize,
        root_of_unity: unsafe{ *(c_settings.expanded_roots_of_unity.add(1)) } ,
        expanded_roots_of_unity: unsafe{std::slice::from_raw_parts(c_settings.expanded_roots_of_unity, c_settings.max_width as usize).to_vec() },
        reverse_roots_of_unity: unsafe{std::slice::from_raw_parts(c_settings.reverse_roots_of_unity, c_settings.max_width as usize).to_vec() },

    }
}

fn fft_settings_to_c(rust_settings : &mut FsFFTSettings) -> CFsFFTSettings{
    let mut roots_of_unity: Vec<FsFr> = rust_settings.expanded_roots_of_unity.clone();
    reverse_bit_order(&mut roots_of_unity);

    CFsFFTSettings{
        max_width: rust_settings.max_width as u64,
        expanded_roots_of_unity: rust_settings.expanded_roots_of_unity.as_mut_ptr(),
        reverse_roots_of_unity: rust_settings.reverse_roots_of_unity.as_mut_ptr(),
        roots_of_unity: roots_of_unity.as_mut_ptr(),
    }
}

fn kzg_settings_to_rust(c_settings : &CFsKzgSettings) -> FsKZGSettings{
    let length = c_settings.fs.max_width as usize;
    FsKZGSettings{
        fs: fft_settings_to_rust(&c_settings.fs),
        secret_g1: unsafe{std::slice::from_raw_parts(c_settings.g1_values, length).to_vec() },
        secret_g2: unsafe{std::slice::from_raw_parts(c_settings.g2_values, length).to_vec() }
    }
}

fn kzg_settings_to_c(rust_settings : &mut FsKZGSettings) -> CFsKzgSettings{
    CFsKzgSettings{
        fs: fft_settings_to_c(&mut rust_settings.fs),
        g1_values: rust_settings.secret_g1.as_mut_ptr(),
        g2_values: rust_settings.secret_g2.as_mut_ptr(),
    }
}

const BLOB_SIZE: usize = 4096;
/// # Safety
///
/// This function should not be called before the horsemen are ready.
#[no_mangle]
pub unsafe extern "C" fn blob_to_kzg_commitment(blob: *const FsFr, s: &CFsKzgSettings) -> FsG1 {
    blob_to_kzg_commitment_rust(std::slice::from_raw_parts(blob, BLOB_SIZE) , &kzg_settings_to_rust(s))
}

// getting *FILE seems impossible 
// https://stackoverflow.com/questions/4862327/is-there-a-way-to-get-the-filename-from-a-file
/// # Safety
///
/// This function should not be called before the horsemen are ready.
#[no_mangle]
pub unsafe extern "C" fn load_trusted_setup_file(out: *mut CFsKzgSettings, inp: *const c_char) {
    let c_str = CStr::from_ptr(inp) ;
    let filename = c_str.to_str().map(|s| s.to_owned()).unwrap();
    let mut settings = load_trusted_setup_rust(&filename);
    *out = kzg_settings_to_c(&mut settings);
}

// #[no_mangle]
// pub extern "C" fn verify_aggregate_kzg_proof(
//     out: *mut bool,
//     blobs: *const u8,
//     expected_kzg_commitments: *const blst_p1,
//     n: usize,
//     kzg_aggregated_proof: *const blst_p1,
//     s: *const FsKZGSettings,
// ) {
//     let mut tmp = unsafe {
//         verify_aggregate_kzg_proof_rust(
//             std::slice::from_raw_parts(blobs, n),
//             std::slice::from_raw_parts(expected_kzg_commitments, n),
//             &FsG1(*kzg_aggregated_proof),
//             &FsKZGSettings(*s),
//         )
//     };
//     unsafe {
//         *out = tmp;
//     }
// }

// fn bytes_to_bls_field(out: *mut BlstFr, bytes: *const u8);
    // fn bytes_from_g1(out: *mut u8, g1: *const BlstP1);
    // fn load_trusted_setup_file(out: *mut KzgKZGSettings4844, inp: *mut FILE) -> KzgRet;
    // fn verify_aggregate_kzg_proof(
    //     out: *mut bool,
    //     blobs: *const u8,
    //     expected_kzg_commitments: *const BlstP1,
    //     n: usize,
    //     kzg_aggregated_proof: *const BlstP1,
    //     s: *const KzgKZGSettings4844,
    // ) -> KzgRet;
    // fn blob_to_kzg_commitment(out: *mut BlstP1, blob: *const u8, s: *const KzgKZGSettings4844);
    // fn compute_aggregate_kzg_proof(
    //     out: *mut BlstP1,
    //     blobs: *const u8,
    //     n: usize,
    //     s: *const KzgKZGSettings4844,
    // ) -> KzgRet;
