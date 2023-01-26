#![allow(non_camel_case_types)]
use crate::data_types::{g1::*, fp::*, fr::*, g2::*};
use crate::kzg_settings::KZGSettings as mKZGSettings;
use crate::fk20_fft::FFTSettings as mFFTSettings;
use crate::eip_4844::FIELD_ELEMENTS_PER_BLOB;
use std::boxed::Box;
use std::slice::from_raw_parts;

pub type limb_t = u64;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct blst_fr {
    pub l: [limb_t; 4usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct blst_fp {
    pub l: [limb_t; 6usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct blst_fp2 {
    pub fp: [blst_fp; 2usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct blst_p1 {
    pub x: blst_fp,
    pub y: blst_fp,
    pub z: blst_fp,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct blst_p2 {
    pub x: blst_fp2,
    pub y: blst_fp2,
    pub z: blst_fp2,
}

pub type g1_t = blst_p1;
pub type g2_t = blst_p2;
pub type fr_t = blst_fr;

const BYTES_PER_COMMITMENT: usize = 48;
const BYTES_PER_PROOF: usize = 48;
const BYTES_PER_FIELD_ELEMENT: usize = 32;
const BYTES_PER_BLOB: usize = 32 * FIELD_ELEMENTS_PER_BLOB;

pub struct Blob {
    pub bytes: [u8; BYTES_PER_BLOB],
}

#[repr(C)]
pub struct KZGCommitment {
    pub bytes: [u8; BYTES_PER_COMMITMENT],
}

#[repr(C)]
pub struct KZGProof {
    pub bytes: [u8; BYTES_PER_PROOF],
}

#[repr(C)]
pub struct BLSFieldElement {
    pub bytes: [u8; BYTES_PER_FIELD_ELEMENT],
}

pub const C_KZG_RET_C_KZG_OK: C_KZG_RET = 0;
pub const C_KZG_RET_C_KZG_BADARGS: C_KZG_RET = 1;
pub const C_KZG_RET_C_KZG_ERROR: C_KZG_RET = 2;
pub const C_KZG_RET_C_KZG_MALLOC: C_KZG_RET = 3;
pub type C_KZG_RET = ::std::os::raw::c_uint;
#[doc = " Stores the setup and parameters needed for performing FFTs."]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FFTSettings {
    #[doc = "< The maximum size of FFT these settings support, a power of 2."]
    pub max_width: u64,
    #[doc = "< Ascending powers of the root of unity, size `width + 1`."]
    pub expanded_roots_of_unity: *mut fr_t,
    #[doc = "< Descending powers of the root of unity, size `width + 1`."]
    pub reverse_roots_of_unity: *mut fr_t,
    #[doc = "< Powers of the root of unity in bit-reversal permutation, size `width`."]
    pub roots_of_unity: *mut fr_t,
}

#[doc = " Stores the setup and parameters needed for computing KZG proofs."]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct KZGSettings {
    #[doc = "< The corresponding settings for performing FFTs"]
    pub fs: *const FFTSettings,
    #[doc = "< G1 group elements from the trusted setup, in Lagrange form bit-reversal permutation"]
    pub g1_values: *mut g1_t,
    #[doc = "< G2 group elements from the trusted setup; both arrays have FIELD_ELEMENTS_PER_BLOB elements"]
    pub g2_values: *mut g2_t,
}


/// # Safety
fn g1_to_cg1(t: &G1,  out: *mut g1_t) {
    unsafe {
        (*out).x.l = t.x.d;
        (*out).y.l = t.y.d;
        (*out).z.l = t.z.d;
    }
}
/// # Safety
fn cg1_to_g1(t: *const g1_t) -> G1 {
    unsafe {
        G1 {
            x: Fp {d: (*t).x.l},
            y: Fp {d: (*t).y.l},
            z: Fp {d: (*t).z.l},
        }
    }
}

// fn fp2_to_cfp2(t: &Fp2,  out: *mut blst_fp2) {
//     unsafe {
//         (*out).x.fp = t.d;
//         (*out).y.fp = t.y.d;
//         (*out).z.fp = t.z.d;
//     }
// }
// fn cg2_to_g2(t: *const g2_t) -> G2 {
//     unsafe {
//         return G1 {
//             x: Fp {d: (*t).x.l},
//             y: Fp {d: (*t).y.l},
//             z: Fp {d: (*t).z.l},
//         };
//     }
// }

// fn g2_to_cg2(t: &G2,  out: *mut g2_t) {
//     unsafe {
//         (*out).x.fp = [blst_fp{l: t.x.d[0].d}, blst_fp{l: t.x.d[1].d}];
//         (*out).y.fp = [blst_fp{l: t.y.d[0].d}, blst_fp{l: t.y.d[1].d}];
//         (*out).z.fp = [blst_fp{l: t.z.d[0].d}, blst_fp{l: t.z.d[1].d}];
//     }
// }
// fn cg2_to_g2(t: *const g2_t) -> G2 {
//     unsafe {
//         return G2 {
//             x: Fp2 {d: [Fp{d: (*t).x.fp[0].l}, Fp{d: (*t).x.fp[1].l}]},
//             y: Fp2 {d: [Fp{d: (*t).y.fp[0].l}, Fp{d: (*t).y.fp[1].l}]},
//             z: Fp2 {d: [Fp{d: (*t).z.fp[0].l}, Fp{d: (*t).z.fp[1].l}]},
//         };
//     }
// }
// fn fr_to_cfr(t: &Fr,  out: *mut fr_t) {
//     unsafe {
//         (*out).l = t.d;
//     }
// }

fn ks_to_cks(t: &mut mKZGSettings, out: *mut KZGSettings) {
    assert_eq!(t.curve.g1_points.len(), t.fft_settings.max_width);
    unsafe {
        (*out).g1_values = t.curve.g1_points.as_mut_ptr() as _;
        (*out).g2_values = t.curve.g2_points.as_mut_ptr() as _;
        let fs = FFTSettings{
            max_width: t.fft_settings.max_width as _,
            roots_of_unity: t.fft_settings.exp_roots_of_unity.as_mut_ptr() as _,
            expanded_roots_of_unity: t.fft_settings.exp_roots_of_unity.as_mut_ptr() as _,
            reverse_roots_of_unity: t.fft_settings.exp_roots_of_unity_rev.as_mut_ptr() as _,
        };
        let b = Box::new(fs);
        (*out).fs = Box::into_raw(b);
    }
}
fn cks_to_ks(t: *const KZGSettings) -> mKZGSettings {
    unsafe {
        crate::fk20_fft::init_globals();
        let fs = (*t).fs;
        let mw = (*fs).max_width as usize;
        let mut ks = mKZGSettings {
            curve: crate::kzg10::Curve {
                g1_gen: G1::gen(),
                g2_gen: G2::gen(), 
                g1_points: Vec::from_raw_parts((*t).g1_values as _, mw, mw),
                g2_points: Vec::from_raw_parts((*t).g2_values as _, 65, 65),
            },
            fft_settings: mFFTSettings {
                max_width: mw,
                root_of_unity: Fr::default(), 
                exp_roots_of_unity: Vec::from_raw_parts((*fs).expanded_roots_of_unity as _, mw+1, mw+1),
                exp_roots_of_unity_rev: Vec::from_raw_parts((*fs).reverse_roots_of_unity as _, mw+1, mw+1),
            },
        };
        ks.fft_settings.root_of_unity = ks.fft_settings.exp_roots_of_unity[1];
        ks
    }

}

// fn cg2_to_g2(t: *const g2_t) -> G2 {
//     unsafe {
//         return G2 {
//             x: Fp2 {d: [Fp{d: (*t).x.fp[0].l}, Fp{d: (*t).x.fp[1].l}]},
//             y: Fp2 {d: [Fp{d: (*t).y.fp[0].l}, Fp{d: (*t).y.fp[1].l}]},
//             z: Fp2 {d: [Fp{d: (*t).z.fp[0].l}, Fp{d: (*t).z.fp[1].l}]},
//         };
//     }
// }

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn bytes_to_g1(out: *mut g1_t, in_: *const u8) -> C_KZG_RET {
    assert!(crate::mcl_methods::init(crate::CurveType::BLS12_381));
    // println!("bytes_to_g1");
    let arr;
    unsafe {
        arr = std::slice::from_raw_parts(in_, 48);
    }
    let t = match crate::eip_4844::bytes_to_g1(arr) {
        Ok(v) => v,
        Err(_) => return C_KZG_RET_C_KZG_BADARGS,
    };

    g1_to_cg1(&t, out);

    C_KZG_RET_C_KZG_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn bytes_from_g1(out: *mut u8, in_: *const g1_t) {
    assert!(crate::mcl_methods::init(crate::CurveType::BLS12_381));
    // println!("bytes_from_g1");

    let b = crate::eip_4844::bytes_from_g1(&cg1_to_g1(in_));
    unsafe {
        let res = std::slice::from_raw_parts_mut(out, b.len()); 
        res.copy_from_slice(&b);
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn load_trusted_setup(
    out: *mut KZGSettings,
    g1_bytes: *const u8,
    n1: usize,
    g2_bytes: *const u8,
    n2: usize,
) -> C_KZG_RET {
    assert!(crate::mcl_methods::init(crate::CurveType::BLS12_381));
    // println!("load_trusted_setup");

    unsafe {
        let mut mks = crate::eip_4844::load_trusted_setup_from_bytes(
            std::slice::from_raw_parts(g1_bytes, n1 * 48),
            std::slice::from_raw_parts(g2_bytes, n2 * 96),
        );
        ks_to_cks(&mut mks, out);
        std::mem::forget(mks);
    }
    C_KZG_RET_C_KZG_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn load_trusted_setup_file(out: *mut KZGSettings, in_: *mut libc::FILE) -> C_KZG_RET {
    assert!(crate::mcl_methods::init(crate::CurveType::BLS12_381));
    // println!("load_trusted_setup_file");
    let mut buf = vec![0u8; 1024 * 1024];
    let len: usize;
    unsafe {
        len = libc::fread(buf.as_mut_ptr() as *mut libc::c_void, 1, buf.len(), in_);
    }
    let s = String::from_utf8(buf[..len].to_vec()).unwrap();

    let (b1, b2) = crate::eip_4844::load_trusted_setup_string(&s);
    let mut mks = crate::eip_4844::load_trusted_setup_from_bytes(b1.as_slice(), b2.as_slice());
    ks_to_cks(&mut mks, out);
    std::mem::forget(mks);

    C_KZG_RET_C_KZG_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn free_trusted_setup(s: *mut KZGSettings) {
    assert!(crate::mcl_methods::init(crate::CurveType::BLS12_381));
    // println!("free_trusted_setup");
    // TODO
    
    drop(cks_to_ks(s));
    let fs = Box::from_raw((*s).fs as *mut FFTSettings);
    drop(fs);
}


unsafe fn  blobs_to_frs(blobs: *const Blob, n: usize) -> (Vec<Vec<Fr>>, bool) {
    let mut ok = true;
    assert_eq!(std::mem::size_of::<Blob>(), BYTES_PER_BLOB);
    let frs = (0..n).map(|i| {
        let (frs, o) = blob_to_frs(blobs.add(i) as _);
    // let frs = from_raw_parts(blobs, n).iter().map(|blob| {
        //let (frs, o) = blob_to_frs(blob);
        ok &= o;
        frs
    }).collect::<Vec<Vec<Fr>>>();
    (frs, ok)
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn compute_aggregate_kzg_proof(
    out: *mut KZGProof,
    blobs: *const Blob,
    n: usize,
    s: *const KZGSettings,
) -> C_KZG_RET {
    assert!(crate::mcl_methods::init(crate::CurveType::BLS12_381));
    // println!("compute_aggregate_kzg_proof");

    let (frs, ok) = blobs_to_frs(blobs, n);
    if !ok {
        return C_KZG_RET_C_KZG_BADARGS
    }

    let ms = cks_to_ks(s); 
    let proof = crate::eip_4844::compute_aggregate_kzg_proof(&frs, &ms);
    std::mem::forget(ms);
    (*out).bytes = crate::eip_4844::bytes_from_g1(&proof);

    C_KZG_RET_C_KZG_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn verify_aggregate_kzg_proof(
    out: *mut bool,
    blobs: *const Blob,
    expected_kzg_commitments: *const KZGCommitment,
    n: usize,
    kzg_aggregated_proof: *const KZGProof,
    s: *const KZGSettings,
) -> C_KZG_RET {
    assert!(crate::mcl_methods::init(crate::CurveType::BLS12_381));
    // println!("verify_aggregate_kzg_proof");

    let (frs, ok) = blobs_to_frs(blobs, n);
    if !ok {
        return C_KZG_RET_C_KZG_BADARGS
    }
    let res: Result<Vec<_>, _> = from_raw_parts(expected_kzg_commitments, n).iter().map(|c| {
        crate::eip_4844::bytes_to_g1(&c.bytes) 
    }).collect();
    let expected = match res {
        Ok(v) => v,
        Err(_) => return C_KZG_RET_C_KZG_BADARGS,
    };
    let proof = match crate::eip_4844::bytes_to_g1(&(*kzg_aggregated_proof).bytes) {
        Ok(v) => v,
        Err(_) => return C_KZG_RET_C_KZG_BADARGS,
    };

    let ms = cks_to_ks(s); 
    *out = crate::eip_4844::verify_aggregate_kzg_proof(&frs, &expected, &proof, &ms);
    std::mem::forget(ms);
    C_KZG_RET_C_KZG_OK
}

unsafe fn blob_to_frs(blob: *const Blob) -> (Vec<Fr>, bool) {
    let mut ok = true;
    let frs = (*blob).bytes.chunks(32).map(|x| {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(x);
        let mut tmp = Fr::default();
        ok &= tmp.deserialize(&bytes);
        tmp
    }).collect::<Vec<Fr>>();

    (frs, ok)
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn blob_to_kzg_commitment(
    out: *mut KZGCommitment,
    blob: *const Blob,
    s: *const KZGSettings,
) -> C_KZG_RET {
    assert!(crate::mcl_methods::init(crate::CurveType::BLS12_381));
    // println!("blob_to_kzg_commitment");

    let (frs, ok) = blob_to_frs(blob);
    if !ok {
        println!("not ok frs");
        return C_KZG_RET_C_KZG_BADARGS
    }
    let ms = cks_to_ks(s); 
    let o = crate::eip_4844::blob_to_kzg_commitment(&frs, &ms);
    (*out).bytes = crate::eip_4844::bytes_from_g1(&o);

    std::mem::forget(ms);

    C_KZG_RET_C_KZG_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn verify_kzg_proof(
    out: *mut bool,
    polynomial_kzg: *const KZGCommitment,
    z: *const [u8; 32],
    y: *const [u8; 32],
    kzg_proof: *const KZGProof,
    s: *const KZGSettings,
) -> C_KZG_RET {
    assert!(crate::mcl_methods::init(crate::CurveType::BLS12_381));
    // println!("verify_kzg_proof");

    let poly = match crate::eip_4844::bytes_to_g1(&(*polynomial_kzg).bytes) {
        Ok(v) => v,
        Err(_) => return C_KZG_RET_C_KZG_BADARGS,
    };

    let fz = crate::eip_4844::bytes_to_bls_field(&*z);
    let fy = crate::eip_4844::bytes_to_bls_field(&*y);
    let proof = match crate::eip_4844::bytes_to_g1(&(*kzg_proof).bytes) {
        Ok(v) => v,
        Err(_) => return C_KZG_RET_C_KZG_BADARGS,
    };


    let ms = cks_to_ks(s); 
    *out = crate::eip_4844::verify_kzg_proof(&poly, &fz, &fy, &proof, &ms);
    std::mem::forget(ms);

    C_KZG_RET_C_KZG_OK
}
