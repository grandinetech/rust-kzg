use ark_bls12_381::G1Affine;
use ark_ec::AffineRepr;
use ark_ff::{BigInteger256, MontFp, PrimeField};

use crate::arkmsm::types::{G1BaseField, G1ScalarField};

// Decompose scalar = q * lambda + r with barret reduction
// Here we implement algorithm 2 example 1 described in
// https://hackmd.io/@chaosma/SyAvcYFxh

const LMDA1: u128 = 0xac45a4010001a402; // lambda high 64 bit
const LMDA0: u128 = 0x00000000ffffffff; // lambda low 64 bit
const INV1: u128 = 0x7c6becf1e01faadd; // 2**256 // lambda - [64, 127] bit
const INV0: u128 = 0x63f6e522f6cfee30; // 2**256 // lambda - [0, 63] bit
const MASK64: u128 = 0xffffffffffffffff;

pub fn decompose(
    scalar: &G1ScalarField,
    window_bits: u32,
) -> (G1ScalarField, G1ScalarField, bool, bool) {
    let (s0, s1, s2, s3, is_neg_scalar) = glv_preprocess_scalar(scalar, window_bits);

    // 255 bits in four 64b limbs
    // let s2: u128 = scalar.into_repr().as_ref()[2] as u128; // 64 bit
    // let s3: u128 = scalar.into_repr().as_ref()[3] as u128; // 63 bit

    // quotient = (scalar_top127b * inv_approx) >> 128, 127 + 129 - 128 = 128b
    let q0: u128 = INV0 * s2;
    let q1: u128 = INV1 * s2 + INV0 * s3 + (q0 >> 64);
    let q2: u128 = s2 + INV1 * s3 + (q1 >> 64);
    let q3: u128 = s3 + (q2 >> 64);

    let mut quotient0: u128 = q2 & MASK64;
    let mut quotient1: u128 = q3 & MASK64;

    // t = quotient * LAMBDA
    let t0: u128 = quotient0 * LMDA0;
    let t1: u128 = quotient1 * LMDA0 + quotient0 * LMDA1 + (t0 >> 64);
    let t2: u128 = quotient1 * LMDA1 + (t1 >> 64);
    // let t3: u128 = t2 >> 64;

    // r = scalar - t
    let mut carry: i128 = s0 as i128 - (t0 & MASK64) as i128;
    let mut r0: u64 = (carry as u128 & MASK64) as u64;
    carry >>= 64;

    carry += s1 as i128 - (t1 & MASK64) as i128;
    let mut r1: u64 = (carry as u128 & MASK64) as u64;
    carry >>= 64;

    carry += s2 as i128 - (t2 & MASK64) as i128;
    let mut r2: u64 = (carry as u128 & MASK64) as u64;

    // remainder is at most 3 * LAMBDA, 130 bit
    // assert_lt!(r2, 4, "remainder at most 130 bit");

    let mut correction = 0u32;
    loop {
        carry = r0 as i128 - LMDA0 as i128;
        let t0: u64 = (carry as u128 & MASK64) as u64;
        carry >>= 64;

        carry += r1 as i128 - LMDA1 as i128;
        let t1: u64 = (carry as u128 & MASK64) as u64;
        carry >>= 64;

        if carry < 0 && r2 == 0 {
            // went negative
            break;
        }

        r2 = (r2 as i128 + carry) as u64;
        r0 = t0;
        r1 = t1;
        correction += 1;
    }

    // correction
    quotient0 += correction as u128;
    quotient1 += quotient0 >> 64;
    quotient0 &= MASK64;

    let mut is_neg_remainder = false;
    if 128 % window_bits == 0 {
        is_neg_remainder = glv_post_processing(&mut quotient0, &mut quotient1, &mut r0, &mut r1);
    }

    (
        G1ScalarField::from(BigInteger256::new([
            quotient0 as u64,
            quotient1 as u64,
            0,
            0,
        ])),
        G1ScalarField::from(BigInteger256::new([r0, r1, 0, 0])),
        is_neg_scalar,
        is_neg_remainder,
    )
}

// With the signed-bucket-index trick, slice[i] add a carry to slice[i+1] when
// MSB of slice[i] is set. If the scalar_bit_length mod by slice_bit_length
// (aka window_size) is zero, an extra slice need to be created for the
// signed-bucket-index trick to work, which introduces performance penalty.
// This happens when window size is 15 or 17 for 255 bits scalars, or window
// size 16 for 128 bits scalars with GLV decomposition
//
// pre and post processing here ensure that MSBs of both quotient and remainder
// are zero, with a trick similar to signed-bucket-index

const R3: i128 = 0x73eda753299d7d48;
const R2: i128 = 0x3339d80809a1d805;
const R1: i128 = 0x53bda402fffe5bfe;
const R0: i128 = 0xffffffff00000001;

// use sP = (N - s)(-P) to make scalar smaller, which ensures scalar MSB is not
// set, and the decomposed phi has MSB unset
fn glv_preprocess_scalar(
    scalar: &G1ScalarField,
    window_bits: u32,
) -> (u128, u128, u128, u128, bool) {
    let mut s = scalar.into_bigint().0;

    let mut is_neg_scalar = false;
    if 128 % window_bits == 0 {
        if s[3] >= 0x3FFFFFFFFFFFFFFF {
            is_neg_scalar = true;

            let mut carry: i128 = 0;
            carry = carry + R0 - s[0] as i128;
            s[0] = (carry as u128 & MASK64) as u64;
            carry >>= 64;

            carry = carry + R1 - s[1] as i128;
            s[1] = (carry as u128 & MASK64) as u64;
            carry >>= 64;

            carry = carry + R2 - s[2] as i128;
            s[2] = (carry as u128 & MASK64) as u64;
            carry >>= 64;

            carry = carry + R3 - s[3] as i128;
            s[3] = (carry as u128 & MASK64) as u64;
        }
        assert!(s[3] < 0x3FFFFFFFFFFFFFFF);
    }

    (
        s[0] as u128,
        s[1] as u128,
        s[2] as u128,
        s[3] as u128,
        is_neg_scalar,
    )
}

// if remainder has MSB set, clear MSB by using labmda - remainder, and add
// carry to quotient
fn glv_post_processing(q0: &mut u128, q1: &mut u128, r0: &mut u64, r1: &mut u64) -> bool {
    if *r1 >= 0x8000000000000000 {
        // add carry to q
        *q0 += 1;
        *q1 += (*q0 == 0) as u128;

        // r = lambda - r
        let mut carry: i128 = 0;
        carry = carry + LMDA0 as i128 - *r0 as i128;
        *r0 = (carry as u128 & MASK64) as u64;
        carry >>= 64;

        carry = carry + LMDA1 as i128 - *r1 as i128;
        *r1 = (carry as u128 & MASK64) as u64;

        assert!(*r1 < 0x8000000000000000);
        return true;
    }
    false
}

const BETA: G1BaseField = MontFp!("4002409555221667392624310435006688643935503118305586438271171395842971157480381377015405980053539358417135540939436");

// lambda * (x, y) = (beta * x, y)
pub fn endomorphism(point: &mut G1Affine) {
    if point.is_zero() {
        return;
    }

    point.x *= BETA;
}
