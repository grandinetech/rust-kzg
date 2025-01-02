use std::{
    ffi::{CStr, CString},
    fs::File,
    io::Read,
    path::PathBuf,
    ptr::null_mut,
};

use kzg::eth::c_bindings::{Blob, Bytes48, CKZGSettings, KZGCommitment, KZGProof};
use kzg::{
    eip_4844::{
        load_trusted_setup_string, BYTES_PER_COMMITMENT, BYTES_PER_FIELD_ELEMENT, BYTES_PER_G1,
        BYTES_PER_G2, BYTES_PER_PROOF,
    },
    eth::c_bindings::CKzgRet,
};
use libc::FILE;

use crate::tests::{
    eip_4844::generate_random_blob_bytes,
    utils::{get_manifest_dir, get_trusted_setup_path},
};

fn get_trusted_setup_fixture_path(fixture: &str) -> String {
    let filename = "trusted_setup_fixture.txt";

    PathBuf::from(get_manifest_dir())
        .join("src/tests/fixtures")
        .join(fixture)
        .join(filename)
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string()
}

fn get_ckzg_settings(
    load_trusted_setup_file: unsafe extern "C" fn(
        out: *mut CKZGSettings,
        in_: *mut FILE,
    ) -> CKzgRet,
) -> CKZGSettings {
    let mut c_settings = CKZGSettings {
        g1_values_lagrange_brp: null_mut(),
        brp_roots_of_unity: null_mut(),
        g1_values_monomial: null_mut(),
        g2_values_monomial: null_mut(),
        reverse_roots_of_unity: null_mut(),
        scratch_size: 0,
        tables: null_mut(),
        wbits: 0,
        x_ext_fft_columns: null_mut(),
        roots_of_unity: null_mut(),
    };

    let trusted_setup_path = CString::new(get_trusted_setup_path()).unwrap();
    let file = unsafe {
        libc::fopen(
            trusted_setup_path.as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"r\0").as_ptr(),
        )
    };
    assert!(!file.is_null());

    let out = unsafe { load_trusted_setup_file(&mut c_settings, file) };

    unsafe {
        libc::fclose(file);
    }

    assert_ne!(out, CKzgRet::BadArgs);

    c_settings
}

pub fn blob_to_kzg_commitment_invalid_blob_test(
    blob_to_kzg_commitment: unsafe extern "C" fn(
        out: *mut KZGCommitment,
        blob: *const Blob,
        s: &CKZGSettings,
    ) -> CKzgRet,
    load_trusted_setup_file: unsafe extern "C" fn(
        out: *mut CKZGSettings,
        in_: *mut FILE,
    ) -> CKzgRet,
) {
    let settings = get_ckzg_settings(load_trusted_setup_file);

    let mut rng = rand::thread_rng();
    let mut blob_bytes = generate_random_blob_bytes(&mut rng);

    let bls_modulus: [u8; BYTES_PER_FIELD_ELEMENT] = [
        0x73, 0xED, 0xA7, 0x53, 0x29, 0x9D, 0x7D, 0x48, 0x33, 0x39, 0xD8, 0x08, 0x09, 0xA1, 0xD8,
        0x05, 0x53, 0xBD, 0xA4, 0x02, 0xFF, 0xFE, 0x5B, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00,
        0x00, 0x01,
    ];
    // Make first field element equal to BLS_MODULUS
    blob_bytes[0..BYTES_PER_FIELD_ELEMENT].copy_from_slice(&bls_modulus);

    let blob = Blob { bytes: blob_bytes };
    let mut commitment = KZGCommitment {
        bytes: [0; BYTES_PER_COMMITMENT],
    };

    let output = unsafe { blob_to_kzg_commitment(&mut commitment, &blob, &settings) };

    assert_eq!(output, CKzgRet::BadArgs)
}

pub fn load_trusted_setup_invalid_g1_byte_length_test(
    load_trusted_setup: unsafe extern "C" fn(
        *mut CKZGSettings,
        *const u8,
        u64,
        *const u8,
        u64,
        *const u8,
        u64,
        u64,
    ) -> CKzgRet,
) {
    let mut file = File::open(get_trusted_setup_path()).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let (mut g1_bytes_monomial, g1_bytes_lagrange, g2_bytes_monomial) =
        load_trusted_setup_string(&contents).unwrap();
    // Add one more point
    let additional = [0; BYTES_PER_G1];
    g1_bytes_monomial.extend_from_slice(&additional);

    let mut loaded_settings = CKZGSettings {
        g1_values_lagrange_brp: null_mut(),
        brp_roots_of_unity: null_mut(),
        g1_values_monomial: null_mut(),
        g2_values_monomial: null_mut(),
        reverse_roots_of_unity: null_mut(),
        scratch_size: 0,
        tables: null_mut(),
        wbits: 0,
        x_ext_fft_columns: null_mut(),
        roots_of_unity: null_mut(),
    };

    let status = unsafe {
        load_trusted_setup(
            &mut loaded_settings,
            g1_bytes_monomial.as_ptr(),
            g1_bytes_monomial.len() as u64,
            g1_bytes_lagrange.as_ptr(),
            g1_bytes_lagrange.len() as u64,
            g2_bytes_monomial.as_ptr(),
            g2_bytes_monomial.len() as u64,
            0,
        )
    };

    assert_eq!(status, CKzgRet::BadArgs)
}

pub fn load_trusted_setup_invalid_g1_point_test(
    load_trusted_setup: unsafe extern "C" fn(
        *mut CKZGSettings,
        *const u8,
        u64,
        *const u8,
        u64,
        *const u8,
        u64,
        u64,
    ) -> CKzgRet,
) {
    let mut file = File::open(get_trusted_setup_path()).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let (mut g1_bytes_monomial, g1_bytes_lagrange, g2_bytes_monomial) =
        load_trusted_setup_string(&contents).unwrap();
    // Break first G1 point
    g1_bytes_monomial[0] = 0;

    let mut loaded_settings = CKZGSettings {
        g1_values_lagrange_brp: null_mut(),
        brp_roots_of_unity: null_mut(),
        g1_values_monomial: null_mut(),
        g2_values_monomial: null_mut(),
        reverse_roots_of_unity: null_mut(),
        scratch_size: 0,
        tables: null_mut(),
        wbits: 0,
        x_ext_fft_columns: null_mut(),
        roots_of_unity: null_mut(),
    };

    let status = unsafe {
        load_trusted_setup(
            &mut loaded_settings,
            g1_bytes_monomial.as_ptr(),
            g1_bytes_monomial.len() as u64,
            g1_bytes_lagrange.as_ptr(),
            g1_bytes_lagrange.len() as u64,
            g2_bytes_monomial.as_ptr(),
            g2_bytes_monomial.len() as u64,
            0,
        )
    };

    assert_eq!(status, CKzgRet::BadArgs)
}

pub fn load_trusted_setup_invalid_g2_byte_length_test(
    load_trusted_setup: unsafe extern "C" fn(
        *mut CKZGSettings,
        *const u8,
        u64,
        *const u8,
        u64,
        *const u8,
        u64,
        u64,
    ) -> CKzgRet,
) {
    let mut file = File::open(get_trusted_setup_path()).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let (g1_bytes_monomial, g1_bytes_lagrange, mut g2_bytes_monomial) =
        load_trusted_setup_string(&contents).unwrap();
    // Add one more point
    let additional = [0; BYTES_PER_G2];
    g2_bytes_monomial.extend_from_slice(&additional);

    let mut loaded_settings = CKZGSettings {
        g1_values_lagrange_brp: null_mut(),
        brp_roots_of_unity: null_mut(),
        g1_values_monomial: null_mut(),
        g2_values_monomial: null_mut(),
        reverse_roots_of_unity: null_mut(),
        scratch_size: 0,
        tables: null_mut(),
        wbits: 0,
        x_ext_fft_columns: null_mut(),
        roots_of_unity: null_mut(),
    };

    let status = unsafe {
        load_trusted_setup(
            &mut loaded_settings,
            g1_bytes_monomial.as_ptr(),
            g1_bytes_monomial.len() as u64,
            g1_bytes_lagrange.as_ptr(),
            g1_bytes_lagrange.len() as u64,
            g2_bytes_monomial.as_ptr(),
            g2_bytes_monomial.len() as u64,
            0,
        )
    };

    assert_eq!(status, CKzgRet::BadArgs)
}

pub fn load_trusted_setup_invalid_g2_point_test(
    load_trusted_setup: unsafe extern "C" fn(
        *mut CKZGSettings,
        *const u8,
        u64,
        *const u8,
        u64,
        *const u8,
        u64,
        u64,
    ) -> CKzgRet,
) {
    let mut file = File::open(get_trusted_setup_path()).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let (g1_bytes_monomial, g1_bytes_lagrange, mut g2_bytes_monomial) =
        load_trusted_setup_string(&contents).unwrap();
    // Break first G2 point
    g2_bytes_monomial[0] = 0;

    let mut loaded_settings = CKZGSettings {
        g1_values_lagrange_brp: null_mut(),
        brp_roots_of_unity: null_mut(),
        g1_values_monomial: null_mut(),
        g2_values_monomial: null_mut(),
        reverse_roots_of_unity: null_mut(),
        scratch_size: 0,
        tables: null_mut(),
        wbits: 0,
        x_ext_fft_columns: null_mut(),
        roots_of_unity: null_mut(),
    };

    let status = unsafe {
        load_trusted_setup(
            &mut loaded_settings,
            g1_bytes_monomial.as_ptr(),
            g1_bytes_monomial.len() as u64,
            g1_bytes_lagrange.as_ptr(),
            g1_bytes_lagrange.len() as u64,
            g2_bytes_monomial.as_ptr(),
            g2_bytes_monomial.len() as u64,
            0,
        )
    };

    assert_eq!(status, CKzgRet::BadArgs)
}

pub fn load_trusted_setup_invalid_form_test(
    load_trusted_setup: unsafe extern "C" fn(
        *mut CKZGSettings,
        *const u8,
        u64,
        *const u8,
        u64,
        *const u8,
        u64,
        u64,
    ) -> CKzgRet,
) {
    let mut file = File::open(get_trusted_setup_fixture_path("old")).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let (g1_bytes_monomial, g1_bytes_lagrange, g2_bytes_monomial) =
        load_trusted_setup_string(&contents).unwrap();

    let mut loaded_settings = CKZGSettings {
        g1_values_lagrange_brp: null_mut(),
        brp_roots_of_unity: null_mut(),
        g1_values_monomial: null_mut(),
        g2_values_monomial: null_mut(),
        reverse_roots_of_unity: null_mut(),
        scratch_size: 0,
        tables: null_mut(),
        wbits: 0,
        x_ext_fft_columns: null_mut(),
        roots_of_unity: null_mut(),
    };

    let status = unsafe {
        load_trusted_setup(
            &mut loaded_settings,
            g1_bytes_monomial.as_ptr(),
            g1_bytes_monomial.len() as u64,
            g1_bytes_lagrange.as_ptr(),
            g1_bytes_lagrange.len() as u64,
            g2_bytes_monomial.as_ptr(),
            g2_bytes_monomial.len() as u64,
            0,
        )
    };

    assert_eq!(status, CKzgRet::BadArgs)
}

pub fn load_trusted_setup_file_invalid_format_test(
    load_trusted_setup_file: unsafe extern "C" fn(
        out: *mut CKZGSettings,
        in_: *mut FILE,
    ) -> CKzgRet,
) {
    struct Fixture {
        name: String,
        message: String,
    }

    let fixtures = [
        Fixture {
            name: "old".to_string(),
            message: "Invalid format because trusted setup is old, i.e. is not in Lagrange form".to_string(),
        },
        Fixture {
            name: "invalid_g1_point_count".to_string(),
            message: "Invalid format because G1 point count in trusted setup file does not match expected".to_string(),
        },
        Fixture {
            name: "invalid_g2_point_count".to_string(),
            message: "Invalid format because G2 point count in trusted setup file does not match expected".to_string(),
        },
        Fixture {
            name: "missing_g1_point_count".to_string(),
            message: "Invalid format because G1 point count is was not found in trusted setup file".to_string(),
        },
        Fixture {
            name: "missing_g2_point_count".to_string(),
            message: "Invalid format because G2 point count is was not found in trusted setup file".to_string(),
        },
        Fixture {
            name: "insufficient_g1_points".to_string(),
            message: "Invalid format because failed to read specified amount of G1 points"
                .to_string(),
        },
        Fixture {
            name: "insufficient_g2_points".to_string(),
            message: "Invalid format because failed to read specified amount of G2 points"
                .to_string(),
        },
        Fixture {
            name: "invalid_chars".to_string(),
            message: "Invalid format because incorrect characters encountered".to_string(),
        },
        Fixture {
            name: "not_a_number".to_string(),
            message: "Invalid format because file starts with not a number".to_string(),
        },
    ];

    for fixture in fixtures {
        let file_path = get_trusted_setup_fixture_path(&fixture.name);
        let file = unsafe {
            let c_file_path = CString::new(file_path.clone()).unwrap();
            libc::fopen(
                c_file_path.as_ptr(),
                CStr::from_bytes_with_nul_unchecked(b"r\0").as_ptr(),
            )
        };

        assert!(!file.is_null());

        let mut loaded_settings = CKZGSettings {
            g1_values_lagrange_brp: null_mut(),
            brp_roots_of_unity: null_mut(),
            g1_values_monomial: null_mut(),
            g2_values_monomial: null_mut(),
            reverse_roots_of_unity: null_mut(),
            scratch_size: 0,
            tables: null_mut(),
            wbits: 0,
            x_ext_fft_columns: null_mut(),
            roots_of_unity: null_mut(),
        };

        let output = unsafe { load_trusted_setup_file(&mut loaded_settings, file) };

        unsafe {
            libc::fclose(file);
        }

        assert!(
            output == CKzgRet::BadArgs,
            "{}, fixture: {file_path}",
            fixture.message
        );
    }
}

pub fn load_trusted_setup_file_valid_format_test(
    load_trusted_setup_file: unsafe extern "C" fn(
        out: *mut CKZGSettings,
        in_: *mut FILE,
    ) -> CKzgRet,
) {
    struct Fixture {
        name: String,
        message: String,
    }

    let fixtures = [
        Fixture {
            name: "valid_whitespace_characters".to_string(),
            message: "Valid format, because whitespace characters must be ignored".to_string(),
        },
        Fixture {
            name: "valid_short_hex".to_string(),
            message: "Valid format, because first character of hex can be omitted, if it is zero (e.g. 07 -> 7)".to_string()
        }
    ];

    for fixture in fixtures {
        let file_path = get_trusted_setup_fixture_path(&fixture.name);
        let file = unsafe {
            let c_file_path = CString::new(file_path.clone()).unwrap();
            libc::fopen(
                c_file_path.as_ptr(),
                CStr::from_bytes_with_nul_unchecked(b"r\0").as_ptr(),
            )
        };

        assert!(!file.is_null());

        let mut loaded_settings = CKZGSettings {
            g1_values_lagrange_brp: null_mut(),
            brp_roots_of_unity: null_mut(),
            g1_values_monomial: null_mut(),
            g2_values_monomial: null_mut(),
            reverse_roots_of_unity: null_mut(),
            scratch_size: 0,
            tables: null_mut(),
            wbits: 0,
            x_ext_fft_columns: null_mut(),
            roots_of_unity: null_mut(),
        };

        let output = unsafe { load_trusted_setup_file(&mut loaded_settings, file) };

        unsafe {
            libc::fclose(file);
        }

        assert!(
            output == CKzgRet::Ok,
            "{}, fixture: {file_path}",
            fixture.message
        );
    }
}

pub fn free_trusted_setup_null_ptr_test(
    free_trusted_setup: unsafe extern "C" fn(s: *mut CKZGSettings) -> (),
) {
    // just should not crash with SIGSEGV
    unsafe {
        free_trusted_setup(null_mut());
    }

    let mut settings = CKZGSettings {
        g1_values_lagrange_brp: null_mut(),
        brp_roots_of_unity: null_mut(),
        g1_values_monomial: null_mut(),
        g2_values_monomial: null_mut(),
        reverse_roots_of_unity: null_mut(),
        scratch_size: 0,
        tables: null_mut(),
        wbits: 0,
        x_ext_fft_columns: null_mut(),
        roots_of_unity: null_mut(),
    };

    // same here, no asserts, just should not crash
    unsafe {
        free_trusted_setup(&mut settings);
    }
}

pub fn free_trusted_setup_set_all_values_to_null_test(
    free_trusted_setup: unsafe extern "C" fn(s: *mut CKZGSettings) -> (),
    load_trusted_setup_file: unsafe extern "C" fn(
        out: *mut CKZGSettings,
        in_: *mut FILE,
    ) -> CKzgRet,
) {
    let mut settings = get_ckzg_settings(load_trusted_setup_file);

    assert!(!settings.g1_values_monomial.is_null());
    assert!(!settings.g1_values_lagrange_brp.is_null());
    assert!(!settings.g2_values_monomial.is_null());
    assert!(!settings.reverse_roots_of_unity.is_null());
    assert!(!settings.roots_of_unity.is_null());
    assert!(!settings.brp_roots_of_unity.is_null());

    unsafe {
        free_trusted_setup(&mut settings);
    };

    assert!(settings.g1_values_monomial.is_null());
    assert!(settings.g1_values_lagrange_brp.is_null());
    assert!(settings.g2_values_monomial.is_null());
    assert!(settings.reverse_roots_of_unity.is_null());
    assert!(settings.roots_of_unity.is_null());
    assert!(settings.brp_roots_of_unity.is_null());
}

pub fn compute_blob_kzg_proof_invalid_blob_test(
    compute_blob_kzg_proof: unsafe extern "C" fn(
        out: *mut KZGProof,
        blob: *const Blob,
        commitment_bytes: *const Bytes48,
        s: &CKZGSettings,
    ) -> CKzgRet,
    load_trusted_setup_file: unsafe extern "C" fn(
        out: *mut CKZGSettings,
        in_: *mut FILE,
    ) -> CKzgRet,
) {
    let settings = get_ckzg_settings(load_trusted_setup_file);

    let mut rng = rand::thread_rng();
    let mut blob_bytes = generate_random_blob_bytes(&mut rng);

    let bls_modulus: [u8; BYTES_PER_FIELD_ELEMENT] = [
        0x73, 0xED, 0xA7, 0x53, 0x29, 0x9D, 0x7D, 0x48, 0x33, 0x39, 0xD8, 0x08, 0x09, 0xA1, 0xD8,
        0x05, 0x53, 0xBD, 0xA4, 0x02, 0xFF, 0xFE, 0x5B, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00,
        0x00, 0x01,
    ];
    // Make first field element equal to BLS_MODULUS
    blob_bytes[0..BYTES_PER_FIELD_ELEMENT].copy_from_slice(&bls_modulus);

    let blob = Blob { bytes: blob_bytes };

    let mut out = KZGProof {
        bytes: [0; BYTES_PER_PROOF],
    };
    let commitment = Bytes48 {
        bytes: [0u8; BYTES_PER_COMMITMENT],
    };

    let out = unsafe { compute_blob_kzg_proof(&mut out, &blob, &commitment, &settings) };

    assert_eq!(out, CKzgRet::BadArgs);
}

pub fn compute_blob_kzg_proof_commitment_is_point_at_infinity_test(
    compute_blob_kzg_proof: unsafe extern "C" fn(
        out: *mut KZGProof,
        blob: *const Blob,
        commitment_bytes: *const Bytes48,
        s: &CKZGSettings,
    ) -> CKzgRet,
    load_trusted_setup_file: unsafe extern "C" fn(
        out: *mut CKZGSettings,
        in_: *mut FILE,
    ) -> CKzgRet,
) {
    let settings = get_ckzg_settings(load_trusted_setup_file);

    let mut rng = rand::thread_rng();
    let blob_bytes = generate_random_blob_bytes(&mut rng);

    let blob = Blob { bytes: blob_bytes };

    let mut out = KZGProof {
        bytes: [0; BYTES_PER_PROOF],
    };

    /* set commitment to point at infinity */
    let mut commitment = Bytes48 {
        bytes: [0; BYTES_PER_COMMITMENT],
    };
    commitment.bytes[0] = 0xc0;

    let out = unsafe { compute_blob_kzg_proof(&mut out, &blob, &commitment, &settings) };

    assert_eq!(out, CKzgRet::Ok);
}

pub fn compute_blob_kzg_proof_zero_input_test(
    compute_blob_kzg_proof: unsafe extern "C" fn(
        out: *mut KZGProof,
        blob: *const Blob,
        commitment_bytes: *const Bytes48,
        s: &CKZGSettings,
    ) -> CKzgRet,
    load_trusted_setup_file: unsafe extern "C" fn(
        out: *mut CKZGSettings,
        in_: *mut FILE,
    ) -> CKzgRet,
) {
    let settings = get_ckzg_settings(load_trusted_setup_file);

    let mut rng = rand::thread_rng();
    let blob_bytes = generate_random_blob_bytes(&mut rng);

    let blob = Blob { bytes: blob_bytes };

    let mut out = KZGProof {
        bytes: [0; BYTES_PER_PROOF],
    };

    /* set commitment to zero */
    let commitment = Bytes48 {
        bytes: [0; BYTES_PER_COMMITMENT],
    };

    let out = unsafe { compute_blob_kzg_proof(&mut out, &blob, &commitment, &settings) };

    assert_eq!(out, CKzgRet::Ok);
}
