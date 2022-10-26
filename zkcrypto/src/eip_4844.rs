use crate::zkfr::blsScalar;

pub fn bytes_to_bls_field(bytes: &[u8; 32usize]) -> blsScalar {
    blsScalar::from_bytes(&bytes).unwrap()
}

pub fn bytes_from_bls_field(fr: &blsScalar) -> [u8; 32usize] {
    fr.to_bytes()
}
