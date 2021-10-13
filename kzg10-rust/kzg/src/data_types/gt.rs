use crate::data_types::fr::Fr;
use crate::data_types::fp::Fp;
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
    fn mclBnGT_isEqual(x: *const GT, y: *const GT) -> i32;
    fn mclBnGT_isZero(x: *const GT) -> i32;
    fn mclBnGT_isOne(x: *const GT) -> i32;

    fn mclBnGT_setStr(x: *mut GT, buf: *const u8, bufSize: usize, ioMode: i32) -> c_int;
    fn mclBnGT_getStr(buf: *mut u8, maxBufSize: usize, x: *const GT, ioMode: i32) -> usize;
    fn mclBnGT_serialize(buf: *mut u8, maxBufSize: usize, x: *const GT) -> usize;
    fn mclBnGT_deserialize(x: *mut GT, buf: *const u8, bufSize: usize) -> usize;

    fn mclBnGT_setInt32(x: *mut GT, v: i32);

    fn mclBnGT_add(z: *mut GT, x: *const GT, y: *const GT);
    fn mclBnGT_sub(z: *mut GT, x: *const GT, y: *const GT);
    fn mclBnGT_neg(y: *mut GT, x: *const GT);

    fn mclBnGT_mul(z: *mut GT, x: *const GT, y: *const GT);
    fn mclBnGT_div(z: *mut GT, x: *const GT, y: *const GT);
    fn mclBnGT_inv(y: *mut GT, x: *const GT);
    fn mclBnGT_sqr(y: *mut GT, x: *const GT);

    fn mclBnGT_pow(z: *mut GT, x: *const GT, y: *const Fr); 
}

#[derive(Default, Debug, Clone)]
#[repr(C)]
pub struct GT {
    d: [Fp; 12],
}
common_impl![GT, mclBnGT_isEqual, mclBnGT_isZero];
serialize_impl![
    GT,
    mlc_methods::mclBn_getFpByteSize() * 12,
    mclBnGT_serialize,
    mclBnGT_deserialize
];
str_impl![GT, 1024, mclBnGT_getStr, mclBnGT_setStr];
int_impl![GT, mclBnGT_setInt32, mclBnGT_isOne];
add_op_impl![GT, mclBnGT_add, mclBnGT_sub, mclBnGT_neg];
field_mul_op_impl![GT, mclBnGT_mul, mclBnGT_div, mclBnGT_inv, mclBnGT_sqr];
impl GT {
    pub fn pow(z: &mut GT, x: &GT, y: &Fr) {
        unsafe { mclBnGT_pow(z, x, y) }
    }
}