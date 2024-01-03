extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::hash::{Hash, Hasher};
use core::ptr::null_mut;
use kzg::common_utils::reverse_bit_order;
use kzg::eip_4844::{
    blob_to_kzg_commitment_raw, compute_blob_kzg_proof_raw, compute_kzg_proof_raw,
    load_trusted_setup_rust, verify_blob_kzg_proof_batch_raw, verify_blob_kzg_proof_raw,
    verify_kzg_proof_raw,
};
use kzg::msm::precompute::PrecomputationTable;
use kzg::{Fr, G1};
#[cfg(feature = "std")]
use libc::FILE;
use siphasher::sip::SipHasher;
#[cfg(feature = "std")]
use std::fs::File;
#[cfg(feature = "std")]
use std::io::Read;

use blst::{blst_fr, blst_p1, blst_p2};

#[cfg(feature = "std")]
use kzg::eip_4844::load_trusted_setup_string;

use kzg::eip_4844::{
    Blob, Bytes32, Bytes48, CKZGSettings, KZGCommitment, KZGProof, BYTES_PER_G1, BYTES_PER_G2,
    C_KZG_RET, C_KZG_RET_BADARGS, C_KZG_RET_OK, FIELD_ELEMENTS_PER_BLOB,
    TRUSTED_SETUP_NUM_G1_POINTS, TRUSTED_SETUP_NUM_G2_POINTS,
};

use crate::types::fft_settings::FsFFTSettings;
use crate::types::fp::FsFp;
use crate::types::fr::FsFr;
use crate::types::g1::{FsG1, FsG1Affine};

use crate::types::g2::FsG2;
use crate::types::kzg_settings::FsKZGSettings;

struct PrecomputationTableManager {
    tables: BTreeMap<u64, Arc<PrecomputationTable<FsFr, FsG1, FsFp, FsG1Affine>>>,
}

impl PrecomputationTableManager {
    pub const fn new() -> Self {
        Self {
            tables: BTreeMap::new(),
        }
    }

    pub fn save_precomputation(&mut self, settings: &mut FsKZGSettings, c_settings: &CKZGSettings) {
        if let Some(precomputation) = settings.precomputation.take() {
            self.tables
                .insert(Self::get_key(c_settings), precomputation);
        }
    }

    pub fn remove_precomputation(&mut self, c_settings: &CKZGSettings) {
        self.tables.remove(&Self::get_key(c_settings));
    }

    pub fn get_precomputation(
        &self,
        c_settings: &CKZGSettings,
    ) -> Option<Arc<PrecomputationTable<FsFr, FsG1, FsFp, FsG1Affine>>> {
        self.tables.get(&Self::get_key(c_settings)).cloned()
    }

    fn get_key(settings: &CKZGSettings) -> u64 {
        let mut hasher = SipHasher::new();
        settings.g1_values.hash(&mut hasher);
        hasher.finish()
    }
}

static mut PRECOMPUTATION_TABLES: PrecomputationTableManager = PrecomputationTableManager::new();

#[cfg(feature = "std")]
pub fn load_trusted_setup_filename_rust(filepath: &str) -> Result<FsKZGSettings, String> {
    let mut file = File::open(filepath).map_err(|_| "Unable to open file".to_string())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|_| "Unable to read file".to_string())?;

    let (g1_bytes, g2_bytes) = load_trusted_setup_string(&contents)?;
    load_trusted_setup_rust(g1_bytes.as_slice(), g2_bytes.as_slice())
}

fn fft_settings_to_rust(c_settings: *const CKZGSettings) -> Result<FsFFTSettings, String> {
    let settings = unsafe { &*c_settings };

    let roots_of_unity = unsafe {
        core::slice::from_raw_parts(settings.roots_of_unity, settings.max_width as usize)
            .iter()
            .map(|r| FsFr(*r))
            .collect::<Vec<FsFr>>()
    };
    let mut expanded_roots_of_unity = roots_of_unity.clone();
    reverse_bit_order(&mut expanded_roots_of_unity)?;
    expanded_roots_of_unity.push(FsFr::one());
    let mut reverse_roots_of_unity = expanded_roots_of_unity.clone();
    reverse_roots_of_unity.reverse();

    let mut first_root = expanded_roots_of_unity[1];
    let first_root_arr = [first_root; 1];
    first_root = first_root_arr[0];

    Ok(FsFFTSettings {
        max_width: settings.max_width as usize,
        root_of_unity: first_root,
        expanded_roots_of_unity,
        reverse_roots_of_unity,
        roots_of_unity,
    })
}

fn kzg_settings_to_rust(c_settings: &CKZGSettings) -> Result<FsKZGSettings, String> {
    let secret_g1 = unsafe {
        core::slice::from_raw_parts(c_settings.g1_values, TRUSTED_SETUP_NUM_G1_POINTS)
            .iter()
            .map(|r| FsG1(*r))
            .collect::<Vec<FsG1>>()
    };
    Ok(FsKZGSettings {
        fs: fft_settings_to_rust(c_settings)?,
        secret_g1,
        secret_g2: unsafe {
            core::slice::from_raw_parts(c_settings.g2_values, TRUSTED_SETUP_NUM_G2_POINTS)
                .iter()
                .map(|r| FsG2(*r))
                .collect::<Vec<FsG2>>()
        },
        precomputation: unsafe { PRECOMPUTATION_TABLES.get_precomputation(c_settings) },
    })
}

fn kzg_settings_to_c(rust_settings: &FsKZGSettings) -> CKZGSettings {
    let g1_val = rust_settings
        .secret_g1
        .iter()
        .map(|r| r.0)
        .collect::<Vec<blst_p1>>();
    let g1_val = Box::new(g1_val);
    let g2_val = rust_settings
        .secret_g2
        .iter()
        .map(|r| r.0)
        .collect::<Vec<blst_p2>>();
    let x = g2_val.into_boxed_slice();
    let stat_ref = Box::leak(x);
    let v = Box::into_raw(g1_val);

    let roots_of_unity = Box::new(
        rust_settings
            .fs
            .roots_of_unity
            .iter()
            .map(|r| r.0)
            .collect::<Vec<blst_fr>>(),
    );

    CKZGSettings {
        max_width: rust_settings.fs.max_width as u64,
        roots_of_unity: unsafe { (*Box::into_raw(roots_of_unity)).as_mut_ptr() },
        g1_values: unsafe { (*v).as_mut_ptr() },
        g2_values: stat_ref.as_mut_ptr(),
    }
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

    let settings = handle_ckzg_badargs!(kzg_settings_to_rust(s));
    let tmp = handle_ckzg_badargs!(blob_to_kzg_commitment_raw((*blob).bytes, &settings));

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
    let mut settings = handle_ckzg_badargs!(load_trusted_setup_rust(g1_bytes, g2_bytes));

    let c_settings = kzg_settings_to_c(&settings);

    PRECOMPUTATION_TABLES.save_precomputation(&mut settings, &c_settings);

    *out = c_settings;
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
    let mut settings = handle_ckzg_badargs!(load_trusted_setup_rust(
        g1_bytes.as_slice(),
        g2_bytes.as_slice()
    ));

    let c_settings = kzg_settings_to_c(&settings);

    PRECOMPUTATION_TABLES.save_precomputation(&mut settings, &c_settings);

    *out = c_settings;

    C_KZG_RET_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn compute_blob_kzg_proof(
    out: *mut KZGProof,
    blob: *const Blob,
    commitment_bytes: *const Bytes48,
    s: &CKZGSettings,
) -> C_KZG_RET {
    let settings = handle_ckzg_badargs!(kzg_settings_to_rust(s));
    let proof = handle_ckzg_badargs!(compute_blob_kzg_proof_raw(
        (*blob).bytes,
        (*commitment_bytes).bytes,
        &settings
    ));

    (*out).bytes = proof.to_bytes();
    C_KZG_RET_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn free_trusted_setup(s: *mut CKZGSettings) {
    if s.is_null() {
        return;
    }

    PRECOMPUTATION_TABLES.remove_precomputation(&*s);

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
    (*s).max_width = 0;
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn verify_kzg_proof(
    ok: *mut bool,
    commitment_bytes: *const Bytes48,
    z_bytes: *const Bytes32,
    y_bytes: *const Bytes32,
    proof_bytes: *const Bytes48,
    s: &CKZGSettings,
) -> C_KZG_RET {
    let settings = handle_ckzg_badargs!(kzg_settings_to_rust(s));

    let result = handle_ckzg_badargs!(verify_kzg_proof_raw(
        (*commitment_bytes).bytes,
        (*z_bytes).bytes,
        (*y_bytes).bytes,
        (*proof_bytes).bytes,
        &settings
    ));

    *ok = result;
    C_KZG_RET_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn verify_blob_kzg_proof(
    ok: *mut bool,
    blob: *const Blob,
    commitment_bytes: *const Bytes48,
    proof_bytes: *const Bytes48,
    s: &CKZGSettings,
) -> C_KZG_RET {
    let settings = handle_ckzg_badargs!(kzg_settings_to_rust(s));

    let result = handle_ckzg_badargs!(verify_blob_kzg_proof_raw(
        (*blob).bytes,
        (*commitment_bytes).bytes,
        (*proof_bytes).bytes,
        &settings,
    ));

    *ok = result;
    C_KZG_RET_OK
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn verify_blob_kzg_proof_batch(
    ok: *mut bool,
    blobs: *const Blob,
    commitments_bytes: *const Bytes48,
    proofs_bytes: *const Bytes48,
    n: usize,
    s: &CKZGSettings,
) -> C_KZG_RET {
    let raw_blobs = core::slice::from_raw_parts(blobs, n)
        .iter()
        .map(|blob| blob.bytes)
        .collect::<Vec<_>>();

    let raw_commitments = core::slice::from_raw_parts(commitments_bytes, n)
        .iter()
        .map(|commitment| commitment.bytes)
        .collect::<Vec<_>>();

    let raw_proofs = core::slice::from_raw_parts(proofs_bytes, n)
        .iter()
        .map(|proof| proof.bytes)
        .collect::<Vec<_>>();

    let settings = match kzg_settings_to_rust(s) {
        Ok(value) => value,
        Err(_) => return C_KZG_RET_BADARGS,
    };

    let result =
        verify_blob_kzg_proof_batch_raw(&raw_blobs, &raw_commitments, &raw_proofs, &settings);

    if let Ok(result) = result {
        *ok = result;
        C_KZG_RET_OK
    } else {
        *ok = false;
        C_KZG_RET_BADARGS
    }
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
    let settings = match kzg_settings_to_rust(s) {
        Ok(value) => value,
        Err(_) => return C_KZG_RET_BADARGS,
    };

    let (proof_out_tmp, fry_tmp) =
        match compute_kzg_proof_raw((*blob).bytes, (*z_bytes).bytes, &settings) {
            Ok(value) => value,
            Err(_) => return C_KZG_RET_BADARGS,
        };

    (*proof_out).bytes = proof_out_tmp.to_bytes();
    (*y_out).bytes = fry_tmp.to_bytes();
    C_KZG_RET_OK
}

#[cfg(test)]
mod tests {
    use kzg_bench::tests::utils::get_trusted_setup_path;

    use crate::eip_4844::{kzg_settings_to_c, kzg_settings_to_rust};

    use super::load_trusted_setup_filename_rust;

    #[test]
    fn kzg_settings_to_rust_check_conversion() {
        let settings = load_trusted_setup_filename_rust(get_trusted_setup_path().as_str());

        assert!(settings.is_ok());

        let settings = settings.unwrap();

        let converted_settings = kzg_settings_to_rust(&kzg_settings_to_c(&settings)).unwrap();

        assert_eq!(
            settings.fs.root_of_unity,
            converted_settings.fs.root_of_unity
        );
        assert_eq!(
            settings.fs.roots_of_unity,
            converted_settings.fs.roots_of_unity
        );
        assert_eq!(
            settings.fs.expanded_roots_of_unity,
            converted_settings.fs.expanded_roots_of_unity
        );
        assert_eq!(
            settings.fs.reverse_roots_of_unity,
            converted_settings.fs.reverse_roots_of_unity
        );
    }
}
