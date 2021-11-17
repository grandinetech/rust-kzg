use crate::consts::{G1_GENERATOR, G2_GENERATOR};
use crate::kzg_proofs::{g2_mul};
use crate::kzg_types::{FsFr, FsG1, FsG2};
use kzg::{Fr, G1, G1Mul, G2Mul};

pub fn is_power_of_two(x: usize) -> bool {
    (x != 0) && ((x & (x - 1)) == 0)
}

pub fn next_power_of_two(v: usize) -> usize {
    let mut v = v;

    v -= 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v |= v >> 32;
    v += 1;
    if v == 0 {
        v += 1
    }

    v
}

pub fn log_2_byte(b: u8) -> usize {
    let mut r = if b > 0xF { 1 } else { 0 } << 2;
    let mut b = b >> r;
    let shift = if b > 0x3 { 1 } else { 0 } << 1;
    b >>= shift + 1;
    r |= shift | b;
    r.into()
}

pub fn log2_pow2(n: usize) -> usize {
    let bytes: [usize; 5] = [0xAAAAAAAA, 0xCCCCCCCC, 0xF0F0F0F0, 0xFF00FF00, 0xFFFF0000];
    let mut r: usize = if (n & bytes[0]) != 0 { 1 } else { 0 };
    r |= (if (n & bytes[1]) != 0 { 1 } else { 0 }) << 1;
    r |= (if (n & bytes[2]) != 0 { 1 } else { 0 }) << 2;
    r |= (if (n & bytes[3]) != 0 { 1 } else { 0 }) << 3;
    r |= (if (n & bytes[4]) != 0 { 1 } else { 0 }) << 4;
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

pub fn min_u64(a: usize, b: usize) -> usize {
    if a < b { a } else { b }
}

pub fn max_u64(a: usize, b: usize) -> usize {
    if a < b { b } else { a }
}

pub fn generate_trusted_setup(n: usize, secret: [u8; 32usize]) -> (Vec<FsG1>, Vec<FsG2>) {
    let s = FsFr::from_scalar(secret);
    let mut s_pow = Fr::one();

    let mut s1 = Vec::new();
    let mut s2 = Vec::new();

    for _ in 0..n {
        s1.push(G1_GENERATOR.mul(&s_pow));
        s2.push(G2_GENERATOR.mul(&s_pow));

        s_pow = s_pow.mul(&s);
    }

    (s1, s2)
}

