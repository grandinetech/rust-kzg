use kzg::{G1, Scalar, Fr};
use crate::kzg_types::{FsFr, FsG2, FsG1};
use crate::consts::{G1_GENERATOR, G2_GENERATOR};
use crate::kzg_proofs::{g1_mul};
use blst::{blst_scalar_from_fr, blst_p2_mult};

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
    v += (v == 0) as usize;

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
    r |= if (n & bytes[1]) != 0 { 1 } else { 0 } << 1;
    r |= if (n & bytes[2]) != 0 { 1 } else { 0 } << 2;
    r |= if (n & bytes[3]) != 0 { 1 } else { 0 } << 3;
    r |= if (n & bytes[4]) != 0 { 1 } else { 0 } << 4;
    r
}

pub fn log2_u64(n: usize) -> usize {
    let mut n2 = n;
    let mut r: usize = 0;
    while (n2 >> 1) != 0 {
        n2 = n2 >> 1;
        r += 1;
    }
    r
}

pub fn min_u64(a: usize, b: usize) -> usize {
    return if a < b {a} else {b};
}

pub fn generate_trusted_setup(s1: &mut Vec<FsG1>, s2: &mut Vec<FsG2>, secret: &Scalar, n: usize) {
    // fr_t s_pow, s;
    // fr_from_scalar(&s, secret);
    // s_pow = fr_one;
    assert!(s1.len() == 0);
    assert!(s2.len() == 0);

    let s = FsFr::from_scalar(secret);
    let mut s_pow = Fr::one();

    for _ in 0..n {
        // g1_mul(s1 + i, &g1_generator, &s_pow);
        // s1[i].mul(G1_GENERATOR, &s_pow);
        let mut tmp_g1 = G1::default();
        g1_mul(&mut tmp_g1, &G1_GENERATOR, &s_pow);
        s1.push(tmp_g1);
        //g2_mul(s2 + i, &g2_generator, &s_pow);
        // s2[i].mul(G2_GENERATOR, &s_pow);
        let tmp_g2 = g2_mul(&G2_GENERATOR, &s_pow);
        s2.push(tmp_g2);
        // fr_mul(&s_pow, &s_pow, &s);
        s_pow = s_pow.mul(&s);
    }
}

fn g2_mul(a: &FsG2, b: &FsFr) -> FsG2 {
    let mut scalar = Scalar::default();
    let mut out = FsG2::default();
    unsafe {
        blst_scalar_from_fr(&mut scalar, &b.0);
        blst_p2_mult(&mut out.0, &a.0, scalar.b.as_ptr() as *const u8, 8 * std::mem::size_of::<Scalar>());
    }
    out
}
