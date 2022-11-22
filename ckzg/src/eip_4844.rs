

use std::{convert::TryInto, fs::File};

use crate::consts::{BlstP1, KzgRet};
use crate::finite::{BlstFr,
};

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
    fn verify_aggregate_kzg_proof(out: *mut bool, blobs: *const *const BlstFr, expected_kzg_commitments: *const BlstP1, n: usize, kzg_aggregated_proof: *const BlstP1, s: *const KzgKZGSettings4844) -> KzgRet;
    fn blob_to_kzg_commitment(out: *mut BlstP1, blob: *const BlstFr, s: *const KzgKZGSettings4844);
    fn compute_aggregate_kzg_proof(out: *mut BlstP1, blobs: *const *const BlstFr, n: usize, s: *const KzgKZGSettings4844) -> KzgRet;
    // fn verify_kzg_proof(out: *mut bool, polynomial_kzg: *const BlstP1, z: *const BlstFr, y: *const BlstFr, kzg_proof: *const BlstP1, s: *const KzgKZGSettings4844) -> KzgRet;
}

pub fn bytes_to_g1_rust(bytes: [u8; 48usize]) -> BlstP1 {
    unsafe{
        let g1: Box<BlstP1> = Box::default();
        let v = Box::<BlstP1>::into_raw(g1);
        bytes_to_g1(v, bytes.as_ptr());
        *Box::<BlstP1>::from_raw(v)
    }
}

pub fn bytes_from_g1_rust(g1: &BlstP1) -> [u8; 48usize] {
    let mut out: [u8; 48usize] = [0; 48];
    unsafe {
        bytes_from_g1(out.as_mut_ptr(), g1);
    }
    out
}

// fn fileChange() {
//     let rust_file = File::open("/etc/passwd").unwrap();
//     unsafe {
//         let c_file = fdopen(
//             rust_file.into_raw_fd(),
//             CStr::from_bytes_with_nul_unchecked(b"r\0").as_ptr(),
//         );
//         let mut buf = [0; 80];
//         let ptr = buf.as_mut_ptr();
//         fgets(ptr, 80, c_file);
//         println!("{}", CStr::from_ptr(ptr).to_str().unwrap());
//         fclose(c_file);
//     }
// }

pub fn load_trusted_setup_rust(filepath: &str) -> KzgKZGSettings4844 {
    // // https://www.reddit.com/r/rust/comments/8sfjp6/converting_between_file_and_stdfsfile/
    let boxed: Box::<KzgKZGSettings4844> = Box::new(KzgKZGSettings4844::default());
    let v = Box::<KzgKZGSettings4844>::into_raw(boxed);
    let res = unsafe{        
        let rust_file = File::open(filepath).unwrap();
        let c_file = fdopen(
            rust_file.into_raw_fd(),
            CStr::from_bytes_with_nul_unchecked(b"r\0").as_ptr(),
        );

        let ret = load_trusted_setup(v, c_file);
        println!("load ret = {:?}", ret);

        *Box::<KzgKZGSettings4844>::from_raw(v)
        // for i in 0..4096{
        //     for j in 0..6{
        //         // let v = (*res.secret_g1.add(i)).x.l[j];
        //         println!("{}", (*res.secret_g1.add(i)).x.l[j]);
        //     }
        //     println!("");
        //     for j in 0..6{
        //         println!("{}", (*res.secret_g1.add(i)).y.l[j]);
        //     }
        //     println!("");
        //     for j in 0..6{
        //         println!("{}", (*res.secret_g1.add(i)).z.l[j]);
        //     }
        //     println!("");
        // }
            
    };
    res

}

// fn fr_batch_inv(out: &mut [BlstFr], a: &[BlstFr], len: usize) {
//     let prod: &mut Vec<BlstFr> = &mut vec![BlstFr::default(); len];
//     let mut i: usize = 1;

//     prod[0] = a[0];

//     while i < len {
//         prod[i] = a[i].mul(&prod[i - 1]);
//         i += 1;
//     }

//     let inv: &mut BlstFr = &mut prod[len - 1].eucl_inverse();

//     i = len - 1;
//     while i > 0 {
//         out[i] = prod[i - 1].mul(inv);
//         *inv = a[i].mul(inv);
//         i -= 1;
//     }
//     out[0] = *inv;
// }

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

    for i in 0..blob.len()
    {
        for j in 0..4{
            println!("{}", blob[i].l[j]);
        }
    }

    // let mut blob_arr: [u8; 131072usize] = [0; 131072usize];
    // let mut it = 0;
    // for i in 0..blob.len(){
    //     for j in 0..4{
    //         let mut part = (1<<8) - 1;
    //         let mut shift = 0;
    //         for k in 0..8{
    //             let mut res = blob[i].l[j]&part;
    //             res = res>>shift;
    //             assert!(res < 256);
    //             blob_arr[it + 7 - k] = (res % 256) as u8;
    //             part = part << 8;
    //             shift += 8;
    //         }
    //         it += 8;
    //     }
    // }
    // assert!(it == 131072);
    println!("blob_to_kzg_commitment fftsettings adresas: {:p}", s.fs);
    unsafe{
        blob_to_kzg_commitment(&mut out, blob.as_ptr(), s);
    }

    println!("outas yra:");
    for j in 0..6{
        // let v = (*res.secret_g1.add(i)).x.l[j];
        println!("{}", out.x.l[j]);
    }
    println!("");
    for j in 0..6{
        println!("{}", out.y.l[j]);
    }
    println!("");
    for j in 0..6{
        println!("{}", out.z.l[j]);
    }
    println!("");

    out
}

// pub fn verify_kzg_proof_rust(
//     polynomial_kzg: &BlstP1,
//     z: &BlstFr,
//     y: &BlstFr,
//     kzg_proof: &BlstP1,
//     s: &KzgKZGSettings,
// ) -> bool {
//     s.check_proof_single(polynomial_kzg, kzg_proof, z, y)
//         .unwrap_or(false)
// }

// pub fn compute_kzg_proof(p: &mut KzgPoly, x: &BlstFr, s: &KzgKZGSettings) -> BlstP1 {
    // todo!();
    // let secret_g1 =
    //     unsafe { slice::from_raw_parts(s.secret_g1, s.length.try_into().unwrap()).to_vec() };

    // assert!(p.len() <= secret_g1.len());

    // let y: BlstFr = evaluate_polynomial_in_evaluation_form(p, x, s);

    // let mut tmp: BlstFr;
    // let mut roots_of_unity: Vec<BlstFr> = unsafe {
    //     slice::from_raw_parts(
    //         (*s.fs).expanded_roots_of_unity,
    //         s.length.try_into().unwrap(),
    //     )
    //     .to_vec()
    // };

    // reverse_bit_order(&mut roots_of_unity);
    // let mut i: usize = 0;
    // let mut m: usize = 0;

    // let mut q: KzgPoly = KzgPoly::new(p.len()).unwrap();

    // let mut inverses_in: Vec<BlstFr> = vec![BlstFr::default(); p.len()];
    // let mut inverses: Vec<BlstFr> = vec![BlstFr::default(); p.len()];

    // while i < q.len() {
    //     if x.equals(&roots_of_unity[i]) {
    //         m = i + 1;
    //         continue;
    //     }

    //     // (p_i - y) / (ω_i - x)
    //     q.set_coeff_at(i, &(p.get_coeff_at(i).sub(&y)));
    //     inverses_in[i] = roots_of_unity[i].sub(x);
    //     i += 1;
    // }

    // fr_batch_inv(&mut inverses, &inverses_in, q.len());

    // i = 0;
    // while i < q.len() {
    //     q.set_coeff_at(i, &q.get_coeff_at(i).mul(&inverses[i]));
    //     i += 1;
    // }

    // if m > 0 {
    //     // ω_m == x
    //     q.set_coeff_at(m, &BlstFr::zero());

    //     m -= 1;
    //     i = 0;
    //     while i < q.len() {
    //         if i == m {
    //             continue;
    //         }
    //         // (p_i - y) * ω_i / (x * (x - ω_i))
    //         tmp = x.sub(&roots_of_unity[i]);
    //         inverses_in[i] = tmp.mul(x);
    //         i += 1;
    //     }
    //     fr_batch_inv(&mut inverses, &inverses_in, q.len());
    //     i = 0;
    //     while i < q.len() {
    //         tmp = (p.get_coeff_at(i)).sub(&y);
    //         tmp = tmp.mul(&roots_of_unity[i]);
    //         tmp = tmp.mul(&inverses[i]);
    //         q.set_coeff_at(m, &(q.get_coeff_at(m)).add(&tmp));
    //         i += 1;
    //     }
    // }

    // g1_lincomb(&secret_g1, q.get_coeffs())
// }

// pub fn evaluate_polynomial_in_evaluation_form_rust(
//     p: &KzgPoly,
//     x: &BlstFr,
//     s: &KzgKZGSettings4844,
// ) -> BlstFr {
//     todo!()

    // let mut tmp: BlstFr = BlstFr::default();
    // unsafe{
    //     evaluate_polynomial_in_evaluation_form(&mut tmp, p, x, s);
    // }

    // let mut inverses_in: Vec<BlstFr> = vec![BlstFr::default(); p.len()];
    // let mut inverses: Vec<BlstFr> = vec![BlstFr::default(); p.len()];
    // let mut i: usize = 0;
    // println!("will slice from raw parts, wish usafe program luck");
    // unsafe{
    //     println!("(*s.fs).max_width = {}", (*s.fs).max_width);
    // }
    // let mut roots_of_unity = unsafe {
    //     slice::from_raw_parts((*s.fs).roots_of_unity, (*s.fs).max_width as usize).to_vec()
    // };

    // println!("roots_of_unity.size() = {}", roots_of_unity.len());

    // // reverse_bit_order(&mut roots_of_unity);

    // while i < p.len() {
    //     if x.equals(&roots_of_unity[i]) {
    //         return p.get_coeff_at(i);
    //     }

    //     inverses_in[i] = x.sub(&roots_of_unity[i]);
    //     i += 1;
    // }
    // fr_batch_inv(&mut inverses, &inverses_in, p.len());

    // let mut out = BlstFr::zero();
    // i = 0;
    // while i < p.len() {
    //     tmp = inverses[i].mul(&roots_of_unity[i]);
    //     tmp = tmp.mul(&p.get_coeff_at(i));
    //     out = out.add(&tmp);
    //     i += 1;
    // }
    // tmp = BlstFr::from_u64(p.len().try_into().unwrap());
    // out = out.div(&tmp).unwrap();
    // tmp = x.pow(p.len());
    // tmp = tmp.sub(&BlstFr::one());
    // out = out.mul(&tmp);
    // out
// }

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
    let mut blob_arr = Vec::<*const BlstFr>::new();
    for i in 0..blobs.len(){
        blob_arr.push(blobs[i].as_ptr());
    }
    unsafe{
        let ret = compute_aggregate_kzg_proof(&mut out, blob_arr.as_ptr(), blobs.len(), ts);
        assert!(ret == KzgRet::KzgOk);
    }
    out
}

pub fn verify_aggregate_kzg_proof_rust(
    blobs: &[Vec<BlstFr>],
    expected_kzg_commitments: &[BlstP1],
    kzg_aggregated_proof: &BlstP1,
    ts: &KzgKZGSettings4844) -> bool {
    let mut out = false;
    let mut blob_arr = Vec::<*const BlstFr>::new();
    for i in 0..blobs.len(){
        blob_arr.push(blobs[i].as_ptr());
    }

    unsafe{
        let ret = verify_aggregate_kzg_proof(&mut out, blob_arr.as_ptr(), expected_kzg_commitments.as_ptr(), blobs.len(), kzg_aggregated_proof as *const BlstP1, ts);
        println!("ret = {:?}", ret);
    }
    // fn verify_aggregate_kzg_proof(out: *mut bool, blobs: *const *const BlstFr, expected_kzg_commitments: *const BlstP1, n: usize, kzg_aggregated_proof: *const BlstFr, s: *const KzgKZGSettings4844) -> KzgRet;
    println!("final out = {}", out);
    out
}
