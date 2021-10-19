use crate::data_types::fr::Fr;
use crate::data_types::fp::Fp;
use crate::data_types::fp2::Fp2;
use std::ops::{Add, AddAssign};
use std::ops::{Div, DivAssign};
use std::ops::{Mul, MulAssign};
use std::ops::{Sub, SubAssign};
use std::mem::MaybeUninit;
use std::os::raw::c_int;
use crate::mcl_methods;

#[link(name = "mcl", kind = "static")]
#[link(name = "mclbn384_256", kind = "static")]
#[link(name = "stdc++")]
#[allow(non_snake_case)]
extern "C" {
    fn mclBnG2_isEqual(x: *const G2, y: *const G2) -> i32;
    fn mclBnG2_isValid(x: *const G2) -> i32;
    fn mclBnG2_isZero(x: *const G2) -> i32;

    fn mclBnG2_setStr(x: *mut G2, buf: *const u8, bufSize: usize, ioMode: i32) -> c_int;
    fn mclBnG2_getStr(buf: *mut u8, maxBufSize: usize, x: *const G2, ioMode: i32) -> usize;
    fn mclBnG2_serialize(buf: *mut u8, maxBufSize: usize, x: *const G2) -> usize;
    fn mclBnG2_deserialize(x: *mut G2, buf: *const u8, bufSize: usize) -> usize;

    fn mclBnG2_add(z: *mut G2, x: *const G2, y: *const G2);
    fn mclBnG2_sub(z: *mut G2, x: *const G2, y: *const G2);
    fn mclBnG2_neg(y: *mut G2, x: *const G2);

    fn mclBnG2_dbl(y: *mut G2, x: *const G2);
    fn mclBnG2_mul(z: *mut G2, x: *const G2, y: *const Fr);
    fn mclBnG2_normalize(y: *mut G2, x: *const G2);
    fn mclBnG2_hashAndMapTo(x: *mut G2, buf: *const u8, bufSize: usize) -> c_int;
}

#[derive(Default, Debug, Clone)]
#[repr(C)]
pub struct G2 {
    pub x: Fp2,
    pub y: Fp2,
    pub z: Fp2,
}
common_impl![G2, mclBnG2_isEqual, mclBnG2_isZero];
is_valid_impl![G2, mclBnG2_isValid];
serialize_impl![
    G2,
    mcl_methods::mclBn_getFpByteSize() * 2,
    mclBnG2_serialize,
    mclBnG2_deserialize
];
str_impl![G2, 1024, mclBnG2_getStr, mclBnG2_setStr];
add_op_impl![G2, mclBnG2_add, mclBnG2_sub, mclBnG2_neg];
ec_impl![
    G2,
    mclBnG2_dbl,
    mclBnG2_mul,
    mclBnG2_normalize,
    mclBnG2_hashAndMapTo
];