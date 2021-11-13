
use crate::data_types::fp::Fp;
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
    fn mclBnFp2_isEqual(x: *const Fp2, y: *const Fp2) -> i32;
    fn mclBnFp2_isZero(x: *const Fp2) -> i32;

    fn mclBnFp2_setStr(x: *mut Fp2, buf: *const u8, bufSize: usize, ioMode: i32) -> c_int;
    fn mclBnFp2_getStr(buf: *mut u8, maxBufSize: usize, x: *const Fp2, ioMode: i32) -> usize;
    fn mclBnFp2_serialize(buf: *mut u8, maxBufSize: usize, x: *const Fp2) -> usize;
    fn mclBnFp2_deserialize(x: *mut Fp2, buf: *const u8, bufSize: usize) -> usize;

    fn mclBnFp2_add(z: *mut Fp2, x: *const Fp2, y: *const Fp2);
    fn mclBnFp2_sub(z: *mut Fp2, x: *const Fp2, y: *const Fp2);
    fn mclBnFp2_neg(y: *mut Fp2, x: *const Fp2);

    fn mclBnFp2_mul(z: *mut Fp2, x: *const Fp2, y: *const Fp2);
    fn mclBnFp2_div(z: *mut Fp2, x: *const Fp2, y: *const Fp2);
    fn mclBnFp2_inv(y: *mut Fp2, x: *const Fp2);
    fn mclBnFp2_sqr(y: *mut Fp2, x: *const Fp2);
    fn mclBnFp2_squareRoot(y: *mut Fp2, x: *const Fp2) -> i32;
}

#[derive(Default, Debug, Clone)]
#[repr(C)]
pub struct Fp2 {
    pub d: [Fp; 2],
}
common_impl![Fp2, mclBnFp2_isEqual, mclBnFp2_isZero];
serialize_impl![
    Fp2,
    mcl_methods::mclBn_getFpByteSize() * 2,
    mclBnFp2_serialize,
    mclBnFp2_deserialize
];
str_impl![Fp2, 1024, mclBnFp2_getStr, mclBnFp2_setStr];
add_op_impl![Fp2, mclBnFp2_add, mclBnFp2_sub, mclBnFp2_neg];
field_mul_op_impl![Fp2, mclBnFp2_mul, mclBnFp2_div, mclBnFp2_inv, mclBnFp2_sqr];
impl Fp2 {
    pub fn square_root(y: &mut Fp2, x: &Fp2) -> bool {
        unsafe { mclBnFp2_squareRoot(y, x) == 0 }
    }
}