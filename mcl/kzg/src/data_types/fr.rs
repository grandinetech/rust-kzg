use crate::mcl_methods;
use primitive_types::U256;
use std::ops::{Add, AddAssign};
use std::ops::{Div, DivAssign};
use std::ops::{Mul, MulAssign};
use std::ops::{Sub, SubAssign};
use std::os::raw::c_int;

#[link(name = "mcl", kind = "static")]
#[link(name = "mclbn384_256", kind = "static")]
#[link(name = "stdc++")]
#[allow(non_snake_case)]
extern "C" {
    fn mclBnFr_isEqual(x: *const Fr, y: *const Fr) -> i32;
    fn mclBnFr_isValid(x: *const Fr) -> i32;
    fn mclBnFr_isZero(x: *const Fr) -> i32;
    fn mclBnFr_isOne(x: *const Fr) -> i32;
    fn mclBnFr_isOdd(x: *const Fr) -> i32;
    fn mclBnFr_isNegative(x: *const Fr) -> i32;

    fn mclBnFr_setStr(x: *mut Fr, buf: *const u8, bufSize: usize, ioMode: i32) -> c_int;
    fn mclBnFr_getStr(buf: *mut u8, maxBufSize: usize, x: *const Fr, ioMode: i32) -> usize;
    fn mclBnFr_serialize(buf: *mut u8, maxBufSize: usize, x: *const Fr) -> usize;
    fn mclBnFr_deserialize(x: *mut Fr, buf: *const u8, bufSize: usize) -> usize;

    fn mclBnFr_setInt32(x: *mut Fr, v: i32);
    fn mclBnFr_setLittleEndian(x: *mut Fr, buf: *const u8, bufSize: usize) -> i32;
    fn mclBnFr_setLittleEndianMod(x: *mut Fr, buf: *const u8, bufSize: usize) -> i32;
    fn mclBnFr_setHashOf(x: *mut Fr, buf: *const u8, bufSize: usize) -> i32;
    fn mclBnFr_setByCSPRNG(x: *mut Fr);

    fn mclBnFr_add(z: *mut Fr, x: *const Fr, y: *const Fr);
    fn mclBnFr_sub(z: *mut Fr, x: *const Fr, y: *const Fr);
    fn mclBnFr_neg(y: *mut Fr, x: *const Fr);

    fn mclBnFr_mul(z: *mut Fr, x: *const Fr, y: *const Fr);
    fn mclBnFr_div(z: *mut Fr, x: *const Fr, y: *const Fr);
    fn mclBnFr_inv(y: *mut Fr, x: *const Fr);
    fn mclBnFr_sqr(y: *mut Fr, x: *const Fr);
    fn mclBnFr_squareRoot(y: *mut Fr, x: *const Fr) -> i32;
}

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct Fr {
    pub d: [u64; crate::MCLBN_FR_UNIT_SIZE],
}

pub struct U2516([u64; 4]);

impl Fr {
    pub fn get_order() -> String {
        mcl_methods::get_curve_order()
    }

    pub fn pow(&self, n: usize) -> Self {
        //No idea if this works
        let mut res = *self;
        for _ in 1 .. n {
            res = res * *self;
        }
        res
    }

    pub fn inverse(&self) -> Self {
        let mut res = Fr::zero();
        Fr::inv(&mut res, self);
        res
    }

    pub fn from_u64_arr(u: &[u64; 4]) -> Self {
        let res = U256([u[0], u[1], u[2], u[3]]);
        Fr::from_str(&res.to_string(), 10).unwrap()
    }

    pub fn from_scalar(secret: &[u8; 32]) -> Self {
        let value0 = u64::from_be_bytes([secret[0], secret[1], secret[2], secret[3], secret[4], secret[5], secret[6], secret[7]]);
        let value1 = u64::from_be_bytes([secret[8], secret[9], secret[10], secret[11], secret[12], secret[13], secret[14], secret[15]]);
        let value2 = u64::from_be_bytes([secret[16], secret[17], secret[18], secret[19], secret[20], secret[21], secret[22], secret[23]]);
        let value3 = u64::from_be_bytes([secret[24], secret[25], secret[26], secret[27], secret[28], secret[29], secret[30], secret[31]]);

        Fr::from_u64_arr(&[value0, value1, value2, value3])
    }
    
    pub fn to_u64_arr(&self) -> [u64; 4] {
        let string = self.get_str(10);
        let num = U256::from_dec_str(&string).unwrap();
        let a = num.0[0];
        let b = num.0[1];
        let c = num.0[2];
        let d = num.0[3];
        [a,b,c,d]
    }
}
common_impl![Fr, mclBnFr_isEqual, mclBnFr_isZero];
is_valid_impl![Fr, mclBnFr_isValid];
serialize_impl![
    Fr,
    mcl_methods::mclBn_getFrByteSize(),
    mclBnFr_serialize,
    mclBnFr_deserialize
];
str_impl![Fr, 1024, mclBnFr_getStr, mclBnFr_setStr];
int_impl![Fr, mclBnFr_setInt32, mclBnFr_isOne];
base_field_impl![
    Fr,
    mclBnFr_setLittleEndian,
    mclBnFr_setLittleEndianMod,
    mclBnFr_setHashOf,
    mclBnFr_setByCSPRNG,
    mclBnFr_isOdd,
    mclBnFr_isNegative,
    mclBnFr_squareRoot
];
add_op_impl![Fr, mclBnFr_add, mclBnFr_sub, mclBnFr_neg];
field_mul_op_impl![Fr, mclBnFr_mul, mclBnFr_div, mclBnFr_inv, mclBnFr_sqr];
