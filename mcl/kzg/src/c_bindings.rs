#![allow(non_camel_case_types)]
use crate::data_types::{fp::*, fr::*, g1::*, g2::*};
use crate::eip_4844::{BYTES_PER_FIELD_ELEMENT, FIELD_ELEMENTS_PER_BLOB};
use crate::fk20_fft::FFTSettings as mFFTSettings;
use crate::kzg10::Polynomial;
use crate::kzg_settings::KZGSettings as mKZGSettings;
use kzg::Poly;
use std::boxed::Box;
use std::convert::TryInto;
use std::slice::{from_raw_parts, from_raw_parts_mut};

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
const BYTES_PER_BLOB: usize = 32 * FIELD_ELEMENTS_PER_BLOB;

#[repr(C)]
pub struct Bytes32 {
    pub bytes: [u8; 32],
}

#[repr(C)]
pub struct Bytes48 {
    pub bytes: [u8; 48],
}

#[repr(C)]
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

pub const C_KZG_RET_C_KZG_OK: C_KZG_RET = 0;
pub const C_KZG_RET_C_KZG_BADARGS: C_KZG_RET = 1;
pub const C_KZG_RET_C_KZG_ERROR: C_KZG_RET = 2;
pub const C_KZG_RET_C_KZG_MALLOC: C_KZG_RET = 3;
pub type C_KZG_RET = ::std::os::raw::c_uint;

#[doc = "Stores the setup and parameters needed for performing FFTs."]
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

#[doc = "Stores the setup and parameters needed for computing KZG proofs."]
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

#[doc = "Internal representation of a polynomial."]
#[repr(C)]
pub struct CPolynomial {
    pub evals: [blst_fr; FIELD_ELEMENTS_PER_BLOB],
}

/// # Safety
unsafe fn cg1_to_g1(t: *const g1_t) -> G1 {
    G1 {
        x: Fp { d: (*t).x.l },
        y: Fp { d: (*t).y.l },
        z: Fp { d: (*t).z.l },
    }
}

unsafe fn ks_to_cks(t: &mut mKZGSettings, out: *mut KZGSettings) {
    assert_eq!(t.curve.g1_points.len(), t.fft_settings.max_width);
    (*out).g1_values = t.curve.g1_points.as_mut_ptr() as _;
    (*out).g2_values = t.curve.g2_points.as_mut_ptr() as _;
    let fs = FFTSettings {
        max_width: t.fft_settings.max_width as _,
        roots_of_unity: t.fft_settings.exp_roots_of_unity.as_mut_ptr() as _,
        expanded_roots_of_unity: t.fft_settings.exp_roots_of_unity.as_mut_ptr() as _,
        reverse_roots_of_unity: t.fft_settings.exp_roots_of_unity_rev.as_mut_ptr() as _,
    };
    let b = Box::new(fs);
    (*out).fs = Box::into_raw(b);
}

unsafe fn cks_to_ks(t: *const KZGSettings) -> mKZGSettings {
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
            exp_roots_of_unity: Vec::from_raw_parts(
                (*fs).expanded_roots_of_unity as _,
                mw + 1,
                mw + 1,
            ),
            exp_roots_of_unity_rev: Vec::from_raw_parts(
                (*fs).reverse_roots_of_unity as _,
                mw + 1,
                mw + 1,
            ),
        },
    };
    ks.fft_settings.root_of_unity = ks.fft_settings.exp_roots_of_unity[1];
    ks
}

unsafe fn deserialize_blob(blob: *const Blob) -> Result<Vec<Fr>, C_KZG_RET> {
    (*blob)
        .bytes
        .chunks(32)
        .map(|x| {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(x);
            let mut tmp = Fr::default();
            let ret = tmp.deserialize(&bytes);
            if !ret {
                Err(C_KZG_RET_C_KZG_BADARGS)
            } else {
                Ok(tmp)
            }
            // fix for `test_verify_kzg_proof_batch__fails_with_incorrect_proof` c-kzg-4844 test
            //if let Ok(fr) = crate::eip_4844::bytes_to_bls_field(&bytes) {
            //    Ok(fr)
            //} else {
            //    Err(C_KZG_RET_C_KZG_BADARGS)
            //}
        })
        .collect::<Result<Vec<Fr>, C_KZG_RET>>()
}

fn cpoly_to_poly(c_poly: &CPolynomial) -> Polynomial {
    let c_poly_coeffs = c_poly.evals;
    let mut poly_rust = Polynomial::new(c_poly_coeffs.len());
    for (pos, e) in c_poly_coeffs.iter().enumerate() {
        poly_rust.set_coeff_at(pos, &Fr { d: e.l });
    }
    poly_rust
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn bytes_from_g1(out: *mut u8, in_: *const g1_t) {
    assert!(crate::mcl_methods::init(crate::CurveType::BLS12_381));
    let b = crate::eip_4844::bytes_from_g1(&cg1_to_g1(in_));
    let res = from_raw_parts_mut(out, b.len());
    res.copy_from_slice(&b);
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
    let mut mks = crate::eip_4844::load_trusted_setup_from_bytes(
        from_raw_parts(g1_bytes, n1 * 48),
        from_raw_parts(g2_bytes, n2 * 96),
    );
    ks_to_cks(&mut mks, out);
    std::mem::forget(mks);
    C_KZG_RET_C_KZG_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn load_trusted_setup_file(
    out: *mut KZGSettings,
    in_: *mut libc::FILE,
) -> C_KZG_RET {
    assert!(crate::mcl_methods::init(crate::CurveType::BLS12_381));

    let mut buf = vec![0u8; 1024 * 1024];
    let len: usize = libc::fread(buf.as_mut_ptr() as *mut libc::c_void, 1, buf.len(), in_);
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
    drop(cks_to_ks(s));
    let fs = Box::from_raw((*s).fs as *mut FFTSettings);
    drop(fs);
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn compute_blob_kzg_proof(
    out: *mut KZGProof,
    blob: *const Blob,
    s: &KZGSettings,
) -> C_KZG_RET {
    assert!(crate::mcl_methods::init(crate::CurveType::BLS12_381));

    let deserialized_blob = deserialize_blob(blob);
    if deserialized_blob.is_err() {
        return deserialized_blob.err().unwrap();
    }
    let ms = cks_to_ks(s);
    let commitment_g1 = crate::eip_4844::compute_blob_kzg_proof(&deserialized_blob.unwrap(), &ms);
    (*out).bytes = crate::eip_4844::bytes_from_g1(&commitment_g1);
    std::mem::forget(ms);

    C_KZG_RET_C_KZG_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn verify_blob_kzg_proof(
    ok: *mut bool,
    blob: *const Blob,
    commitment_bytes: *const Bytes48,
    proof_bytes: *const Bytes48,
    s: &KZGSettings,
) -> C_KZG_RET {
    assert!(crate::mcl_methods::init(crate::CurveType::BLS12_381));

    let deserialized_blob = deserialize_blob(blob);
    if deserialized_blob.is_err() {
        return deserialized_blob.err().unwrap();
    }
    let commitment_g1 = crate::eip_4844::bytes_to_g1(&(*commitment_bytes).bytes);
    let proof_g1 = crate::eip_4844::bytes_to_g1(&(*proof_bytes).bytes);
    if commitment_g1.is_err() || proof_g1.is_err() {
        return C_KZG_RET_C_KZG_BADARGS;
    }

    let ms = cks_to_ks(s);
    *ok = crate::eip_4844::verify_blob_kzg_proof(
        &deserialized_blob.unwrap(),
        &commitment_g1.unwrap(),
        &proof_g1.unwrap(),
        &ms,
    );
    std::mem::forget(ms);

    C_KZG_RET_C_KZG_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn verify_blob_kzg_proof_batch(
    ok: *mut bool,
    blobs: *const Blob,
    commitments_bytes: *const Bytes48,
    proofs_bytes: *const Bytes48,
    n: usize,
    s: &KZGSettings,
) -> C_KZG_RET {
    let mut deserialized_blobs: Vec<Vec<Fr>> = Vec::new();
    let mut commitments_g1: Vec<G1> = Vec::new();
    let mut proofs_g1: Vec<G1> = Vec::new();

    let raw_blobs = from_raw_parts(blobs, n);
    let raw_commitments = from_raw_parts(commitments_bytes, n);
    let raw_proofs = from_raw_parts(proofs_bytes, n);

    for i in 0..n {
        let deserialized_blob = deserialize_blob(&raw_blobs[i]);
        if deserialized_blob.is_err() {
            return deserialized_blob.err().unwrap();
        }

        let commitment_g1 = crate::eip_4844::bytes_to_g1(&raw_commitments[i].bytes);
        let proof_g1 = crate::eip_4844::bytes_to_g1(&raw_proofs[i].bytes);
        if commitment_g1.is_err() || proof_g1.is_err() {
            return C_KZG_RET_C_KZG_BADARGS;
        }

        deserialized_blobs.push(deserialized_blob.unwrap());
        commitments_g1.push(commitment_g1.unwrap());
        proofs_g1.push(proof_g1.unwrap());
    }

    let ms = cks_to_ks(s);
    *ok = crate::eip_4844::verify_blob_kzg_proof_batch(
        &deserialized_blobs,
        &commitments_g1,
        &proofs_g1,
        &ms,
    );
    std::mem::forget(ms);

    C_KZG_RET_C_KZG_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn blob_to_kzg_commitment(
    out: *mut KZGCommitment,
    blob: *const Blob,
    s: *const KZGSettings,
) -> C_KZG_RET {
    assert!(crate::mcl_methods::init(crate::CurveType::BLS12_381));

    let deserialized_blob = deserialize_blob(blob);
    if deserialized_blob.is_err() {
        return deserialized_blob.err().unwrap();
    }
    let ms = cks_to_ks(s);
    let tmp = crate::eip_4844::blob_to_kzg_commitment(&deserialized_blob.unwrap(), &ms);
    (*out).bytes = crate::eip_4844::bytes_from_g1(&tmp);
    std::mem::forget(ms);

    C_KZG_RET_C_KZG_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn verify_kzg_proof(
    out: *mut bool,
    commitment_bytes: *const Bytes48,
    z_bytes: *const Bytes32,
    y_bytes: *const Bytes32,
    proof_bytes: *const Bytes48,
    s: *const KZGSettings,
) -> C_KZG_RET {
    assert!(crate::mcl_methods::init(crate::CurveType::BLS12_381));

    let frz = crate::eip_4844::bytes_to_bls_field(&(*z_bytes).bytes);
    let fry = crate::eip_4844::bytes_to_bls_field(&(*y_bytes).bytes);
    let g1commitment = crate::eip_4844::bytes_to_g1(&(*commitment_bytes).bytes);
    let g1proof = crate::eip_4844::bytes_to_g1(&(*proof_bytes).bytes);

    if frz.is_err() || fry.is_err() || g1commitment.is_err() || g1proof.is_err() {
        return C_KZG_RET_C_KZG_BADARGS;
    }

    let ms = cks_to_ks(s);
    *out = crate::eip_4844::verify_kzg_proof(
        &g1commitment.unwrap(),
        &frz.unwrap(),
        &fry.unwrap(),
        &g1proof.unwrap(),
        &ms,
    );
    std::mem::forget(ms);

    C_KZG_RET_C_KZG_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn compute_kzg_proof(
    out: *mut KZGProof,
    blob: *const Blob,
    z_bytes: *const Bytes32,
    s: *const KZGSettings,
) -> C_KZG_RET {
    assert!(crate::mcl_methods::init(crate::CurveType::BLS12_381));

    let deserialized_blob = deserialize_blob(blob);
    if deserialized_blob.is_err() {
        return deserialized_blob.err().unwrap();
    }
    let frz = crate::eip_4844::bytes_to_bls_field(&(*z_bytes).bytes);
    if frz.is_err() {
        return frz.err().unwrap() as C_KZG_RET;
    }
    let ms = cks_to_ks(s);
    let tmp = crate::eip_4844::compute_kzg_proof(&deserialized_blob.unwrap(), &frz.unwrap(), &ms);
    (*out).bytes = crate::eip_4844::bytes_from_g1(&tmp);
    std::mem::forget(ms);

    C_KZG_RET_C_KZG_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn evaluate_polynomial_in_evaluation_form(
    out: *mut blst_fr,
    p: &CPolynomial,
    x: &blst_fr,
    s: &KZGSettings,
) -> C_KZG_RET {
    assert!(crate::mcl_methods::init(crate::CurveType::BLS12_381));

    let poly = cpoly_to_poly(p);
    let frx = Fr { d: x.l };
    let ms = cks_to_ks(s);
    let result = crate::eip_4844::evaluate_polynomial_in_evaluation_form(&poly, &frx, &ms);
    *out = blst_fr { l: result.d };
    std::mem::forget(ms);

    C_KZG_RET_C_KZG_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn bytes_to_bls_field(out: *mut blst_fr, b: &Bytes32) -> C_KZG_RET {
    assert!(crate::mcl_methods::init(crate::CurveType::BLS12_381));
    let fr = crate::eip_4844::bytes_to_bls_field(&b.bytes);
    if fr.is_err() {
        return fr.err().unwrap() as C_KZG_RET;
    }
    *out = blst_fr { l: fr.unwrap().d };
    C_KZG_RET_C_KZG_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn blob_to_polynomial(p: *mut CPolynomial, blob: *const Blob) -> C_KZG_RET {
    assert!(crate::mcl_methods::init(crate::CurveType::BLS12_381));

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        let start = i * BYTES_PER_FIELD_ELEMENT;
        let bytes_array: [u8; BYTES_PER_FIELD_ELEMENT] = (*blob).bytes
            [start..(start + BYTES_PER_FIELD_ELEMENT)]
            .try_into()
            .unwrap();
        let bytes = Bytes32 { bytes: bytes_array };
        let fr = crate::eip_4844::bytes_to_bls_field(&bytes.bytes);
        if fr.is_err() {
            return fr.err().unwrap() as C_KZG_RET;
        }
        (*p).evals[i] = blst_fr { l: fr.unwrap().d };
    }

    C_KZG_RET_C_KZG_OK
}
