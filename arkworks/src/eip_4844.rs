extern crate alloc;

use blst::{blst_p1, blst_p2, blst_fr};
use kzg::common_utils::{load_trusted_setup_rust, reverse_bit_order, verify_kzg_proof_rust, verify_blob_kzg_proof_rust, fr_batch_inv};
use crate::kzg_proofs::{KZGSettings, FFTSettings};
use crate::kzg_types::{ArkFr, ArkG1, ArkG2};
use kzg::eip_4844::{
    load_trusted_setup_string, CKZGSettings, TRUSTED_SETUP_NUM_G1_POINTS, TRUSTED_SETUP_NUM_G2_POINTS, Blob, C_KZG_RET, BYTES_PER_FIELD_ELEMENT, C_KZG_RET_BADARGS, KZGCommitment, FIELD_ELEMENTS_PER_BLOB, C_KZG_RET_OK, BYTES_PER_G1, BYTES_PER_G2, KZGProof, Bytes48, Bytes32,
};
use kzg::{FFTSettings as FFTSettingsT, KZGSettings as LKZGSettings, Fr, blob_to_kzg_commitment_rust, G1, compute_blob_kzg_proof_rust, cfg_into_iter, verify_blob_kzg_proof_batch_rust, compute_kzg_proof_rust, Poly};
use std::ptr::null_mut;

#[cfg(feature = "std")]
use libc::FILE;
#[cfg(feature = "std")]
use std::fs::File;
#[cfg(feature = "std")]
use std::io::Read;

#[cfg(feature = "parallel")]
use rayon::prelude::*;
use crate::utils::PolyData;

#[cfg(feature = "std")]
pub fn load_trusted_setup_filename_rust(filepath: &str) -> Result<KZGSettings, String> {
    let mut file = File::open(filepath).map_err(|_| "Unable to open file".to_string())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|_| "Unable to read file".to_string())?;

    let (g1_bytes, g2_bytes) = load_trusted_setup_string(&contents)?;
    load_trusted_setup_rust(
        g1_bytes.as_slice(),
        g2_bytes.as_slice()
    )
}

fn fft_settings_to_rust(c_settings: *const CKZGSettings) -> Result<FFTSettings, String> {
    let settings = unsafe { &*c_settings };
    let roots_of_unity = unsafe {
        core::slice::from_raw_parts(settings.roots_of_unity, settings.max_width as usize)
            .iter()
            .map(|r| ArkFr::from_blst_fr(*r))
            .collect::<Vec<ArkFr>>()
    };
    let mut expanded_roots_of_unity = roots_of_unity.clone();
    reverse_bit_order(&mut expanded_roots_of_unity)?;
    expanded_roots_of_unity.push(ArkFr::one());
    let mut reverse_roots_of_unity = expanded_roots_of_unity.clone();
    reverse_roots_of_unity.reverse();

    let mut first_root = expanded_roots_of_unity[1];
    let first_root_arr = [first_root; 1];
    first_root = first_root_arr[0];

    Ok(FFTSettings {
        max_width: settings.max_width as usize,
        root_of_unity: first_root,
        expanded_roots_of_unity,
        reverse_roots_of_unity,
        roots_of_unity,
    })
}

fn kzg_settings_to_rust(c_settings: &CKZGSettings) -> Result<KZGSettings, String> {
    let secret_g1 = unsafe {
        core::slice::from_raw_parts(c_settings.g1_values, TRUSTED_SETUP_NUM_G1_POINTS)
            .iter()
            .map(|r| ArkG1::from_blst_p1(*r))
            .collect::<Vec<ArkG1>>()
    };
    let secret_g2 = unsafe {
            core::slice::from_raw_parts(c_settings.g2_values, TRUSTED_SETUP_NUM_G2_POINTS)
                .iter()
                .map(|r| ArkG2::from_blst_p2(*r))
                .collect::<Vec<ArkG2>>()
    };
    Ok(KZGSettings {
        fs: fft_settings_to_rust(c_settings)?,
        secret_g1,
        secret_g2
    })
}

fn kzg_settings_to_c(rust_settings: &KZGSettings) -> CKZGSettings {
    let g1_val = rust_settings
        .secret_g1
        .iter()
        .map(|r| r.to_blst_p1())
        .collect::<Vec<blst_p1>>();
    let g1_val = Box::new(g1_val);
    let g2_val = rust_settings
        .secret_g2
        .iter()
        .map(|r| r.to_blst_p2())
        .collect::<Vec<blst_p2>>();
    let x = g2_val.into_boxed_slice();
    let stat_ref = Box::leak(x);
    let v = Box::into_raw(g1_val);

    let roots_of_unity = Box::new(
        rust_settings
            .fs
            .roots_of_unity
            .iter()
            .map(|r| r.to_blst_fr())
            .collect::<Vec<blst_fr>>(),
    );

    CKZGSettings {
        max_width: rust_settings.fs.max_width as u64,
        roots_of_unity: unsafe { (*Box::into_raw(roots_of_unity)).as_mut_ptr() },
        g1_values: unsafe { (*v).as_mut_ptr() },
        g2_values: stat_ref.as_mut_ptr(),
    }
}

unsafe fn deserialize_blob(blob: *const Blob) -> Result<Vec<ArkFr>, C_KZG_RET> {
    (*blob)
        .bytes
        .chunks(BYTES_PER_FIELD_ELEMENT)
        .map(|chunk| {
            let mut bytes = [0u8; BYTES_PER_FIELD_ELEMENT];
            bytes.copy_from_slice(chunk);
            if let Ok(result) = ArkFr::from_bytes(&bytes) {
                Ok(result)
            } else {
                Err(C_KZG_RET_BADARGS)
            }
        })
        .collect::<Result<Vec<ArkFr>, C_KZG_RET>>()
}

pub fn evaluate_polynomial_in_evaluation_form_rust(
    p: &PolyData,
    x: &ArkFr,
    s: &KZGSettings,
) -> Result<ArkFr, String> {
    if p.coeffs.len() != FIELD_ELEMENTS_PER_BLOB {
        return Err(String::from("Incorrect field elements count."));
    }

    let roots_of_unity: &Vec<ArkFr> = &s.fs.roots_of_unity;
    let mut inverses_in: Vec<ArkFr> = vec![ArkFr::default(); FIELD_ELEMENTS_PER_BLOB];
    let mut inverses: Vec<ArkFr> = vec![ArkFr::default(); FIELD_ELEMENTS_PER_BLOB];

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        if x.equals(&roots_of_unity[i]) {
            return Ok(p.get_coeff_at(i));
        }
        inverses_in[i] = x.sub(&roots_of_unity[i]);
    }

    fr_batch_inv(&mut inverses, &inverses_in, FIELD_ELEMENTS_PER_BLOB)?;

    let mut tmp: ArkFr;
    let mut out = ArkFr::zero();

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        tmp = inverses[i].mul(&roots_of_unity[i]);
        tmp = tmp.mul(&p.coeffs[i]);
        out = out.add(&tmp);
    }

    tmp = ArkFr::from_u64(FIELD_ELEMENTS_PER_BLOB as u64);
    out = match out.div(&tmp) {
        Ok(value) => value,
        Err(err) => return Err(err),
    };
    tmp = x.pow(FIELD_ELEMENTS_PER_BLOB);
    tmp = tmp.sub(&ArkFr::one());
    out = out.mul(&tmp);
    Ok(out)
}

pub fn blob_to_polynomial_rust(blob: &[ArkFr]) -> Result<PolyData, String> {
    if blob.len() != FIELD_ELEMENTS_PER_BLOB {
        return Err(String::from("Blob length must be FIELD_ELEMENTS_PER_BLOB"));
    }
    let mut p: PolyData = PolyData::new(FIELD_ELEMENTS_PER_BLOB);
    p.coeffs = blob.to_vec();
    Ok(p)
}

macro_rules! handle_ckzg_badargs {
    ($x: expr) => {
        match $x {
            Ok(value) => value,
            Err(_) => return C_KZG_RET_BADARGS,
        }
    };
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn blob_to_kzg_commitment(
    out: *mut KZGCommitment,
    blob: *const Blob,
    s: &CKZGSettings,
) -> C_KZG_RET {
    if TRUSTED_SETUP_NUM_G1_POINTS == 0 {
        // FIXME: load_trusted_setup should set this value, but if not, it fails
        TRUSTED_SETUP_NUM_G1_POINTS = FIELD_ELEMENTS_PER_BLOB
    };

    let deserialized_blob = handle_ckzg_badargs!(deserialize_blob(blob));
    let settings = handle_ckzg_badargs!(kzg_settings_to_rust(s));
    let tmp = handle_ckzg_badargs!(blob_to_kzg_commitment_rust(&deserialized_blob, &settings));

    (*out).bytes = tmp.to_bytes();
    C_KZG_RET_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn load_trusted_setup(
    out: *mut CKZGSettings,
    g1_bytes: *const u8,
    n1: usize,
    g2_bytes: *const u8,
    n2: usize,
) -> C_KZG_RET {
    let g1_bytes = core::slice::from_raw_parts(g1_bytes, n1 * BYTES_PER_G1);
    let g2_bytes = core::slice::from_raw_parts(g2_bytes, n2 * BYTES_PER_G2);
    TRUSTED_SETUP_NUM_G1_POINTS = g1_bytes.len() / BYTES_PER_G1;
    let settings = handle_ckzg_badargs!(
    load_trusted_setup_rust(
        g1_bytes,
        g2_bytes
    ));

    *out = kzg_settings_to_c(&settings);
    C_KZG_RET_OK
}

/// # Safety
#[cfg(feature = "std")]
#[no_mangle]
pub unsafe extern "C" fn load_trusted_setup_file(
    out: *mut CKZGSettings,
    in_: *mut FILE,
) -> C_KZG_RET {
    let mut buf = vec![0u8; 1024 * 1024];
    let len: usize = libc::fread(buf.as_mut_ptr() as *mut libc::c_void, 1, buf.len(), in_);
    let s = handle_ckzg_badargs!(String::from_utf8(buf[..len].to_vec()));
    let (g1_bytes, g2_bytes) = handle_ckzg_badargs!(load_trusted_setup_string(&s));
    TRUSTED_SETUP_NUM_G1_POINTS = g1_bytes.len() / BYTES_PER_G1;
    if TRUSTED_SETUP_NUM_G1_POINTS != FIELD_ELEMENTS_PER_BLOB {
        // Helps pass the Java test "shouldThrowExceptionOnIncorrectTrustedSetupFromFile",
        // as well as 5 others that pass only if this one passes (likely because Java doesn't
        // deallocate its KZGSettings pointer when no exception is thrown).
        return C_KZG_RET_BADARGS;
    }
    let settings = handle_ckzg_badargs!(load_trusted_setup_rust(
        g1_bytes.as_slice(),
        g2_bytes.as_slice()
    ));

    *out = kzg_settings_to_c(&settings);
    C_KZG_RET_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn free_trusted_setup(s: *mut CKZGSettings) {
    if s.is_null() {
        return;
    }

    let max_width = (*s).max_width as usize;
    let roots = Box::from_raw(core::slice::from_raw_parts_mut(
        (*s).roots_of_unity,
        max_width,
    ));
    drop(roots);
    (*s).roots_of_unity = null_mut();

    let g1 = Box::from_raw(core::slice::from_raw_parts_mut(
        (*s).g1_values,
        TRUSTED_SETUP_NUM_G1_POINTS,
    ));
    drop(g1);
    (*s).g1_values = null_mut();

    let g2 = Box::from_raw(core::slice::from_raw_parts_mut(
        (*s).g2_values,
        TRUSTED_SETUP_NUM_G2_POINTS,
    ));
    drop(g2);
    (*s).g2_values = null_mut();
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn compute_blob_kzg_proof(
    out: *mut KZGProof,
    blob: *const Blob,
    commitment_bytes: *mut Bytes48,
    s: &CKZGSettings,
) -> C_KZG_RET {
    let deserialized_blob = match deserialize_blob(blob) {
        Ok(value) => value,
        Err(err) => return err,
    };

    let commitment_g1 = handle_ckzg_badargs!(ArkG1::from_bytes(&(*commitment_bytes).bytes));
    let settings = handle_ckzg_badargs!(kzg_settings_to_rust(s));
    let proof = handle_ckzg_badargs!(compute_blob_kzg_proof_rust(
        &deserialized_blob,
        &commitment_g1,
        &settings
    ));

    (*out).bytes = proof.to_bytes();
    C_KZG_RET_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn compute_kzg_proof(
    proof_out: *mut KZGProof,
    y_out: *mut Bytes32,
    blob: *const Blob,
    z_bytes: *const Bytes32,
    s: &CKZGSettings,
) -> C_KZG_RET {
    let deserialized_blob = match deserialize_blob(blob) {
        Ok(value) => value,
        Err(err) => return err,
    };

    let frz = match ArkFr::from_bytes(&(*z_bytes).bytes) {
        Ok(value) => value,
        Err(_) => return C_KZG_RET_BADARGS,
    };

    let settings = match kzg_settings_to_rust(s) {
        Ok(value) => value,
        Err(_) => return C_KZG_RET_BADARGS,
    };

    let (proof_out_tmp, fry_tmp) = match compute_kzg_proof_rust(&deserialized_blob, &frz, &settings)
    {
        Ok(value) => value,
        Err(_) => return C_KZG_RET_BADARGS,
    };

    (*proof_out).bytes = proof_out_tmp.to_bytes();
    (*y_out).bytes = fry_tmp.to_bytes();
    C_KZG_RET_OK
}

