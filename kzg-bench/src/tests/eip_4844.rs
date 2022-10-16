use kzg::Fr;

// Tests taken from https://github.com/dankrad/c-kzg/blob/4844/min-bindings/python/tests.py

// Simple test of compute_powers
pub fn compute_powers_test<TFr: Fr>
( 
    compute_powers: &dyn Fn(&TFr, usize) -> Vec<TFr>,
    bytes_to_bls_field: &dyn Fn(&mut TFr, [u8; 32usize])
) 
{
    let x: u64 = 32930439;
    let n = 11;

    let mut x_bls = TFr::default();
    
    let mut x_bytes: [u8; 32] = [0; 32];
    x_bytes[..8].copy_from_slice(&x.to_le_bytes());
    
    bytes_to_bls_field(&mut x_bls, x_bytes);
    
    let powers = compute_powers(&x_bls, n);

    let mut p_check: [u64; 4] = [1, 0, 0, 0];
    let module: u128 = 1 << 64;
    
    for p in powers {
        assert_eq!(p_check, p.to_u64_arr());
        for i in (0..4).rev() {
            let tmp = p_check[i] as u128 * x as u128;
            p_check[i] = (tmp % module) as u64;
            if i != 3 {
                p_check[i + 1] += (tmp >> 64) as u64;
            }
        }
    }
}
