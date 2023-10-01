extern crate alloc;

use alloc::vec::Vec;

use kzg::{Fr, G1Mul, G2Mul};

use crate::consts::{G1_GENERATOR, G2_GENERATOR};
use crate::types::fr::FsFr;
use crate::types::g1::FsG1;
use crate::types::g2::FsG2;
use alloc::string::String;

pub fn log_2_byte(b: u8) -> usize {
    let mut r = u8::from(b > 0xF) << 2;
    let mut b = b >> r;
    let shift = u8::from(b > 0x3) << 1;
    b >>= shift + 1;
    r |= shift | b;
    r.into()
}

pub fn log2_pow2(n: usize) -> usize {
    let bytes: [usize; 5] = [0xAAAAAAAA, 0xCCCCCCCC, 0xF0F0F0F0, 0xFF00FF00, 0xFFFF0000];
    let mut r: usize = usize::from((n & bytes[0]) != 0);
    r |= usize::from((n & bytes[1]) != 0) << 1;
    r |= usize::from((n & bytes[2]) != 0) << 2;
    r |= usize::from((n & bytes[3]) != 0) << 3;
    r |= usize::from((n & bytes[4]) != 0) << 4;
    r
}

pub fn log2_u64(n: usize) -> usize {
    let mut n2 = n;
    let mut r: usize = 0;
    while (n2 >> 1) != 0 {
        n2 >>= 1;
        r += 1;
    }
    r
}

pub fn generate_trusted_setup(n: usize, secret: [u8; 32usize]) -> (Vec<FsG1>, Vec<FsG2>) {
    let s = FsFr::hash_to_bls_field(secret);
    let mut s_pow = Fr::one();

    let mut s1 = Vec::with_capacity(n);
    let mut s2 = Vec::with_capacity(n);

    for _ in 0..n {
        s1.push(G1_GENERATOR.mul(&s_pow));
        s2.push(G2_GENERATOR.mul(&s_pow));

        s_pow = s_pow.mul(&s);
    }

    (s1, s2)
}

pub fn reverse_bit_order<T>(values: &mut [T]) -> Result<(), String>
where
    T: Clone,
{
    if values.is_empty() {
        return Err(String::from("Values can not be empty"));
    }

    // does not match with c-kzg implementation, but required for internal tests
    if values.len() == 1 {
        return Ok(());
    }

    if !values.len().is_power_of_two() {
        return Err(String::from("Values length has to be a power of 2"));
    }

    let unused_bit_len = values.len().leading_zeros() + 1;

    for i in 0..values.len() {
        let r = i.reverse_bits() >> unused_bit_len;
        if r > i {
            let tmp = values[r].clone();
            values[r] = values[i].clone();
            values[i] = tmp;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::reverse_bit_order;

    #[test]
    fn reverse_bit_order_bad_arguments() {
        // empty array should fail
        assert!(reverse_bit_order(&mut [0u8; 0]).is_err());
        // array with 1 element should be ignored
        assert!(reverse_bit_order(&mut [1u8]).is_ok());
        // array with 3 elements should fail, because 3 is not a power of 2
        assert!(reverse_bit_order(&mut [1u8, 2u8, 3u8]).is_err());
        // array with 4 elements should pass
        assert!(reverse_bit_order(&mut [1u8, 2u8, 3u8, 4u8]).is_ok());
    }
}
