#![allow(non_camel_case_types)]
use crate::Vec;
pub use blst::{blst_fr, blst_p1, blst_p2};
use core::ffi::c_uint;
use sha2::{Digest, Sha256};

////////////////////////////// Constant values for EIP-4844 //////////////////////////////

pub const FIELD_ELEMENTS_PER_BLOB: usize = if cfg!(feature = "minimal-spec") {
    4
} else {
    4096
};

pub const BYTES_PER_G1: usize = 48;
pub const BYTES_PER_G2: usize = 96;
pub const BYTES_PER_BLOB: usize = BYTES_PER_FIELD_ELEMENT * FIELD_ELEMENTS_PER_BLOB;
pub const BYTES_PER_FIELD_ELEMENT: usize = 32;
pub const BYTES_PER_PROOF: usize = 48;
pub const BYTES_PER_COMMITMENT: usize = 48;

pub const TRUSTED_SETUP_PATH: &str = if cfg!(feature = "minimal-spec") {
    "src/trusted_setups/trusted_setup_4.txt"
} else {
    "src/trusted_setups/trusted_setup.txt"
};

// Currently, we only support fixed amount of G1 and G2 points contained in trusted setups.
// Issue arises when a binding using the C API loads different G1 point quantities each time.
pub static mut TRUSTED_SETUP_NUM_G1_POINTS: usize = 0;

pub const TRUSTED_SETUP_NUM_G2_POINTS: usize = 65;

pub const CHALLENGE_INPUT_SIZE: usize =
    FIAT_SHAMIR_PROTOCOL_DOMAIN.len() + 16 + BYTES_PER_BLOB + BYTES_PER_COMMITMENT;

pub const FIAT_SHAMIR_PROTOCOL_DOMAIN: [u8; 16] = [
    70, 83, 66, 76, 79, 66, 86, 69, 82, 73, 70, 89, 95, 86, 49, 95,
]; // "FSBLOBVERIFY_V1_"

pub const RANDOM_CHALLENGE_KZG_BATCH_DOMAIN: [u8; 16] = [
    82, 67, 75, 90, 71, 66, 65, 84, 67, 72, 95, 95, 95, 86, 49, 95,
]; // "RCKZGBATCH___V1_"

////////////////////////////// C API for EIP-4844 //////////////////////////////

pub type C_KZG_RET = c_uint;

pub const C_KZG_RET_OK: C_KZG_RET = 0;
pub const C_KZG_RET_BADARGS: C_KZG_RET = 1;
pub const C_KZG_RET_ERROR: C_KZG_RET = 2;
pub const C_KZG_RET_MALLOC: C_KZG_RET = 3;

#[repr(C)]
pub struct Bytes32 {
    pub bytes: [u8; 32],
}

#[repr(C)]
pub struct Bytes48 {
    pub bytes: [u8; 48],
}

#[repr(C)]
pub struct BLSFieldElement {
    pub bytes: [u8; BYTES_PER_FIELD_ELEMENT],
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

#[repr(C)]
pub struct CFFTSettings {
    pub max_width: u64,
    pub expanded_roots_of_unity: *mut blst_fr,
    pub reverse_roots_of_unity: *mut blst_fr,
    pub roots_of_unity: *mut blst_fr,
}

#[repr(C)]
pub struct CKZGSettings {
    pub fs: *const CFFTSettings,
    pub g1_values: *mut blst_p1,
    pub g2_values: *mut blst_p2,
}

////////////////////////////// Utility functions for EIP-4844 //////////////////////////////

pub fn load_trusted_setup_string(contents: &str) -> (Vec<u8>, Vec<u8>) {
    let mut lines = contents.lines();
    let length = lines.next().unwrap().parse::<usize>().unwrap();
    let n2 = lines.next().unwrap().parse::<usize>().unwrap();

    let g1_bytes = (0..length)
        .flat_map(|_| {
            let line = lines.next().unwrap();
            assert_eq!(line.len(), 96);
            (0..line.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&line[i..i + 2], 16).unwrap())
                .collect::<Vec<u8>>()
        })
        .collect::<Vec<u8>>();

    let g2_bytes = (0..n2)
        .flat_map(|_| {
            let line = lines.next().unwrap();
            assert_eq!(line.len(), 192);
            (0..line.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&line[i..i + 2], 16).unwrap())
                .collect::<Vec<u8>>()
        })
        .collect::<Vec<u8>>();

    (g1_bytes, g2_bytes)
}

pub fn bytes_of_uint64(out: &mut [u8], mut n: u64) {
    for byte in out.iter_mut().rev().take(8) {
        *byte = (n & 0xff) as u8;
        n >>= 8;
    }
}

pub fn hash(x: &[u8]) -> [u8; 32] {
    Sha256::digest(x).into()
}

#[macro_export]
macro_rules! cfg_into_iter {
    ($e: expr) => {{
        #[cfg(feature = "parallel")]
        let result = $e.into_par_iter();

        #[cfg(not(feature = "parallel"))]
        let result = $e.into_iter();

        result
    }};
}
