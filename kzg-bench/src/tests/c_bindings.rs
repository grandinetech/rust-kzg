use std::{fs::File, io::Read, path::PathBuf, ptr::null_mut};

use kzg::eip_4844::{
    load_trusted_setup_string, Blob, CKZGSettings, KZGCommitment, BYTES_PER_COMMITMENT,
    BYTES_PER_FIELD_ELEMENT, BYTES_PER_G1, BYTES_PER_G2, C_KZG_RET, C_KZG_RET_BADARGS,
    C_KZG_RET_OK,
};

use crate::tests::{
    eip_4844::generate_random_blob_bytes,
    utils::{get_manifest_dir, get_trusted_setup_path},
};

pub fn blob_to_kzg_commitment_invalid_blob_test(
    load_trusted_setup: unsafe extern "C" fn(
        *mut CKZGSettings,
        *const u8,
        usize,
        *const u8,
        usize,
    ) -> C_KZG_RET,
    blob_to_kzg_commitment: unsafe extern "C" fn(
        out: *mut KZGCommitment,
        blob: *const Blob,
        s: &CKZGSettings,
    ) -> C_KZG_RET,
) {
    let mut file = File::open(get_trusted_setup_path())
        .map_err(|_| {})
        .unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let (g1_bytes, g2_bytes) = load_trusted_setup_string(&contents).unwrap();

    let mut c_settings = CKZGSettings {
        g1_values: null_mut(),
        g2_values: null_mut(),
        max_width: 0,
        roots_of_unity: null_mut(),
    };

    let status = unsafe {
        load_trusted_setup(
            &mut c_settings,
            g1_bytes.as_ptr(),
            g1_bytes.len() / BYTES_PER_G1,
            g2_bytes.as_ptr(),
            g2_bytes.len() / BYTES_PER_G2,
        )
    };
    assert_eq!(status, C_KZG_RET_OK);

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

    let output = unsafe { blob_to_kzg_commitment(&mut commitment, &blob, &c_settings) };

    assert_eq!(output, C_KZG_RET_BADARGS)
}

pub fn load_trusted_setup_invalid_g1_byte_length_test(
    load_trusted_setup: unsafe extern "C" fn(
        *mut CKZGSettings,
        *const u8,
        usize,
        *const u8,
        usize,
    ) -> C_KZG_RET,
) {
    let mut file = File::open(get_trusted_setup_path()).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let (mut g1_bytes, g2_bytes) = load_trusted_setup_string(&contents).unwrap();
    // Add one more point
    let additional = [0; BYTES_PER_G1];
    g1_bytes.extend_from_slice(&additional);

    let mut loaded_settings = CKZGSettings {
        g1_values: null_mut(),
        g2_values: null_mut(),
        max_width: 0,
        roots_of_unity: null_mut(),
    };

    let status = unsafe {
        load_trusted_setup(
            &mut loaded_settings,
            g1_bytes.as_ptr(),
            g1_bytes.len() / BYTES_PER_G1,
            g2_bytes.as_ptr(),
            g2_bytes.len() / BYTES_PER_G2,
        )
    };

    assert_eq!(status, C_KZG_RET_BADARGS)
}

pub fn load_trusted_setup_invalid_g1_point_test(
    load_trusted_setup: unsafe extern "C" fn(
        *mut CKZGSettings,
        *const u8,
        usize,
        *const u8,
        usize,
    ) -> C_KZG_RET,
) {
    let mut file = File::open(get_trusted_setup_path()).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let (mut g1_bytes, g2_bytes) = load_trusted_setup_string(&contents).unwrap();
    // Break first G1 point
    g1_bytes[0] = 0;

    let mut loaded_settings = CKZGSettings {
        g1_values: null_mut(),
        g2_values: null_mut(),
        max_width: 0,
        roots_of_unity: null_mut(),
    };

    let status = unsafe {
        load_trusted_setup(
            &mut loaded_settings,
            g1_bytes.as_ptr(),
            g1_bytes.len() / BYTES_PER_G1,
            g2_bytes.as_ptr(),
            g2_bytes.len() / BYTES_PER_G2,
        )
    };

    assert_eq!(status, C_KZG_RET_BADARGS)
}

pub fn load_trusted_setup_invalid_g2_byte_length_test(
    load_trusted_setup: unsafe extern "C" fn(
        *mut CKZGSettings,
        *const u8,
        usize,
        *const u8,
        usize,
    ) -> C_KZG_RET,
) {
    let mut file = File::open(get_trusted_setup_path()).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let (g1_bytes, mut g2_bytes) = load_trusted_setup_string(&contents).unwrap();
    // Add one more point
    let additional = [0; BYTES_PER_G2];
    g2_bytes.extend_from_slice(&additional);

    let mut loaded_settings = CKZGSettings {
        g1_values: null_mut(),
        g2_values: null_mut(),
        max_width: 0,
        roots_of_unity: null_mut(),
    };

    let status = unsafe {
        load_trusted_setup(
            &mut loaded_settings,
            g1_bytes.as_ptr(),
            g1_bytes.len() / BYTES_PER_G1,
            g2_bytes.as_ptr(),
            g2_bytes.len() / BYTES_PER_G2,
        )
    };

    assert_eq!(status, C_KZG_RET_BADARGS)
}

pub fn load_trusted_setup_invalid_g2_point_test(
    load_trusted_setup: unsafe extern "C" fn(
        *mut CKZGSettings,
        *const u8,
        usize,
        *const u8,
        usize,
    ) -> C_KZG_RET,
) {
    let mut file = File::open(get_trusted_setup_path()).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let (g1_bytes, mut g2_bytes) = load_trusted_setup_string(&contents).unwrap();
    // Break first G2 point
    g2_bytes[0] = 0;

    let mut loaded_settings = CKZGSettings {
        g1_values: null_mut(),
        g2_values: null_mut(),
        max_width: 0,
        roots_of_unity: null_mut(),
    };

    let status = unsafe {
        load_trusted_setup(
            &mut loaded_settings,
            g1_bytes.as_ptr(),
            g1_bytes.len() / BYTES_PER_G1,
            g2_bytes.as_ptr(),
            g2_bytes.len() / BYTES_PER_G2,
        )
    };

    assert_eq!(status, C_KZG_RET_BADARGS)
}

pub fn load_trusted_setup_invalid_form_test(
    load_trusted_setup: unsafe extern "C" fn(
        *mut CKZGSettings,
        *const u8,
        usize,
        *const u8,
        usize,
    ) -> C_KZG_RET,
) {
    let trusted_setup_name = if cfg!(feature = "minimal-spec") {
        "trusted_setup_4_old.txt"
    } else {
        "trusted_setup_old.txt"
    };
    let mut file = File::open(
        PathBuf::from(get_manifest_dir())
            .join("src/tests/fixtures")
            .join(trusted_setup_name)
            .as_os_str()
            .to_str()
            .unwrap(),
    )
    .unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let (g1_bytes, g2_bytes) = load_trusted_setup_string(&contents).unwrap();

    let mut loaded_settings = CKZGSettings {
        g1_values: null_mut(),
        g2_values: null_mut(),
        max_width: 0,
        roots_of_unity: null_mut(),
    };

    let status = unsafe {
        load_trusted_setup(
            &mut loaded_settings,
            g1_bytes.as_ptr(),
            g1_bytes.len() / BYTES_PER_G1,
            g2_bytes.as_ptr(),
            g2_bytes.len() / BYTES_PER_G2,
        )
    };

    assert_eq!(status, C_KZG_RET_BADARGS)
}
