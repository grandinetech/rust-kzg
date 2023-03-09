const FIELD_ELEMENTS_PER_BLOB: usize = 4096;
const BYTES_PER_BLOB: usize = 32 * FIELD_ELEMENTS_PER_BLOB;

#[repr(C)]
pub struct Blob {
    pub bytes: [u8; BYTES_PER_BLOB],
}

#[repr(C)]
pub struct KZGCommitment {
    pub bytes: [u8; 48],
}

#[repr(C)]
pub struct KZGProof {
    pub bytes: [u8; 48],
}

#[repr(C)]
pub struct BLSFieldElement {
    pub bytes: [u8; 32],
}
