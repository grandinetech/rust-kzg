use std::ops::{Add, AddAssign};
use std::ops::{Div, DivAssign};
use std::ops::{Mul, MulAssign};
use std::ops::{Sub, SubAssign};
use std::os::raw::c_int;
use crate::mcl_methods::*;

#[link(name = "mcl", kind = "static")]
#[link(name = "mclbn384_256", kind = "static")]
#[link(name = "stdc++")]
#[allow(non_snake_case)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
extern "C" {
    fn mclBnFp_isEqual(x: *const Fp, y: *const Fp) -> i32;
    fn mclBnFp_isValid(x: *const Fp) -> i32;
    fn mclBnFp_isZero(x: *const Fp) -> i32;
    fn mclBnFp_isOne(x: *const Fp) -> i32;
    fn mclBnFp_isOdd(x: *const Fp) -> i32;
    fn mclBnFp_isNegative(x: *const Fp) -> i32;

    fn mclBnFp_setStr(x: *mut Fp, buf: *const u8, bufSize: usize, ioMode: i32) -> c_int;
    fn mclBnFp_getStr(buf: *mut u8, maxBufSize: usize, x: *const Fp, ioMode: i32) -> usize;
    fn mclBnFp_serialize(buf: *mut u8, maxBufSize: usize, x: *const Fp) -> usize;
    fn mclBnFp_deserialize(x: *mut Fp, buf: *const u8, bufSize: usize) -> usize;

    fn mclBnFp_setInt32(x: *mut Fp, v: i32);
    fn mclBnFp_setLittleEndian(x: *mut Fp, buf: *const u8, bufSize: usize) -> i32;
    fn mclBnFp_setLittleEndianMod(x: *mut Fp, buf: *const u8, bufSize: usize) -> i32;
    fn mclBnFp_setHashOf(x: *mut Fp, buf: *const u8, bufSize: usize) -> i32;
    fn mclBnFp_setByCSPRNG(x: *mut Fp);

    fn mclBnFp_add(z: *mut Fp, x: *const Fp, y: *const Fp);
    fn mclBnFp_sub(z: *mut Fp, x: *const Fp, y: *const Fp);
    fn mclBnFp_neg(y: *mut Fp, x: *const Fp);

    fn mclBnFp_mul(z: *mut Fp, x: *const Fp, y: *const Fp);
    fn mclBnFp_div(z: *mut Fp, x: *const Fp, y: *const Fp);
    fn mclBnFp_inv(y: *mut Fp, x: *const Fp);
    fn mclBnFp_sqr(y: *mut Fp, x: *const Fp);
    fn mclBnFp_squareRoot(y: *mut Fp, x: *const Fp) -> i32;

}

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct Fp {
    pub d: [u64; crate::MCLBN_FP_UNIT_SIZE],
}
impl Fp {
    pub fn get_order() -> String {
        get_field_order()
    }
}
common_impl![Fp, mclBnFp_isEqual, mclBnFp_isZero];
is_valid_impl![Fp, mclBnFp_isValid];
serialize_impl![
    Fp,
    mclBn_getFpByteSize(),
    mclBnFp_serialize,
    mclBnFp_deserialize
];
str_impl![Fp, 1024, mclBnFp_getStr, mclBnFp_setStr];
int_impl![Fp, mclBnFp_setInt32, mclBnFp_isOne];
base_field_impl![
    Fp,
    mclBnFp_setLittleEndian,
    mclBnFp_setLittleEndianMod,
    mclBnFp_setHashOf,
    mclBnFp_setByCSPRNG,
    mclBnFp_isOdd,
    mclBnFp_isNegative,
    mclBnFp_squareRoot
];
unsafe{
    add_op_impl![Fp, mclBnFp_add, mclBnFp_sub, mclBnFp_neg];
}
field_mul_op_impl![Fp, mclBnFp_mul, mclBnFp_div, mclBnFp_inv, mclBnFp_sqr];
