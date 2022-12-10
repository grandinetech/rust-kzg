use crate::kzg_types::FsFr;

pub fn bytes_from_bls_field(fr: &FsFr) -> [u8; 32usize] {
    fr.to_scalar()
}

pub fn bytes_to_bls_field(bytes: &[u8; 32usize]) -> FsFr {
    FsFr::from_scalar(*bytes)
}
