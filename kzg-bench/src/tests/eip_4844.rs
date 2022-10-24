use kzg::*;

// Tests taken from https://github.com/dankrad/c-kzg/blob/4844/min-bindings/python/tests.py

pub fn bytes_to_bls_field_test<TFr: Fr>(bytes_to_bls_field: &dyn Fn(&[u8; 32usize]) -> TFr) {
    let x: u64 = 329;

    let mut x_bytes: [u8; 32] = [0; 32];
    x_bytes[..8].copy_from_slice(&x.to_le_bytes());

    let x_bls = bytes_to_bls_field(&x_bytes);

    assert_eq!(x, x_bls.to_u64_arr()[0]);
}
