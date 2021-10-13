use crate::data_types::fr::Fr;
use crate::data_types::fp::Fp;
use crate::data_types::fp2::Fp2;
use std::ops::{Add, AddAssign};
use std::ops::{Div, DivAssign};
use std::ops::{Mul, MulAssign};
use std::ops::{Sub, SubAssign};
use std::mem::MaybeUninit;
use std::os::raw::c_int;
use crate::mlc_methods;

#[link(name = "mcl", kind = "static")]
#[link(name = "mclbn384_256", kind = "static")]
#[link(name = "stdc++")]
#[allow(non_snake_case)]
extern "C" {
    pub fn mclBnG1_isEqual(x: *const G1, y: *const G1) -> i32;
    pub fn mclBnG1_isValid(x: *const G1) -> i32;
    pub fn mclBnG1_isZero(x: *const G1) -> i32;

    pub fn mclBnG1_setStr(x: *mut G1, buf: *const u8, bufSize: usize, ioMode: i32) -> c_int;
    pub fn mclBnG1_getStr(buf: *mut u8, maxBufSize: usize, x: *const G1, ioMode: i32) -> usize;
    pub fn mclBnG1_serialize(buf: *mut u8, maxBufSize: usize, x: *const G1) -> usize;
    pub fn mclBnG1_deserialize(x: *mut G1, buf: *const u8, bufSize: usize) -> usize;
    pub fn mclBnG1_mulVec(x: *mut G1, vec1: *const G1, vec2: *const Fr, bufSize: usize);

    pub fn mclBnG1_add(z: *mut G1, x: *const G1, y: *const G1);
    pub fn mclBnG1_sub(z: *mut G1, x: *const G1, y: *const G1);
    pub fn mclBnG1_neg(y: *mut G1, x: *const G1);

    pub fn mclBnG1_dbl(y: *mut G1, x: *const G1);
    pub fn mclBnG1_mul(z: *mut G1, x: *const G1, y: *const Fr);
    pub fn mclBnG1_normalize(y: *mut G1, x: *const G1);
    pub fn mclBnG1_hashAndMapTo(x: *mut G1, buf: *const u8, bufSize: usize) -> c_int;
}

#[derive(Default, Debug, Clone)]
#[repr(C)]
pub struct G1 {
    pub x: Fp,
    pub y: Fp,
    pub z: Fp,
}
common_impl![G1, mclBnG1_isEqual, mclBnG1_isZero];
is_valid_impl![G1, mclBnG1_isValid];
serialize_impl![
    G1,
    mlc_methods::mclBn_getFpByteSize(),
    mclBnG1_serialize,
    mclBnG1_deserialize
];
str_impl![G1, 1024, mclBnG1_getStr, mclBnG1_setStr];
add_op_impl![G1, mclBnG1_add, mclBnG1_sub, mclBnG1_neg];
ec_impl![
    G1,
    mclBnG1_dbl,
    mclBnG1_mul,
    mclBnG1_normalize,
    mclBnG1_hashAndMapTo
];