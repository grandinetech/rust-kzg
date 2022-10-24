use mcl_rust::eip_4844::*;
use mcl_rust::CurveType;
use mcl_rust::mcl_methods::init;
use kzg_bench::tests::eip_4844::*;

#[test]
pub fn test_bytes_to_bls_field() {
    assert!(init(CurveType::BLS12_381));
    bytes_to_bls_field_test(&bytes_to_bls_field);
}
