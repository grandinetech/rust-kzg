use std::{cmp::min, iter, mem::{MaybeUninit}, ops, usize, vec};
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
    // global functions
    fn mclBn_init(curve: c_int, compiledTimeVar: c_int) -> c_int;
    fn mclBn_getVersion() -> u32;
    fn mclBn_getFrByteSize() -> u32;
    fn mclBn_getFpByteSize() -> u32;
    fn mclBn_getCurveOrder(buf: *mut u8, maxBufSize: usize) -> usize;
    fn mclBn_getFieldOrder(buf: *mut u8, maxBufSize: usize) -> usize;
    fn mclBn_pairing(z: *mut GT, x: *const G1, y: *const G2);
    fn mclBn_millerLoop(z: *mut GT, x: *const G1, y: *const G2);
    fn mclBn_finalExp(y: *mut GT, x: *const GT);
    fn mclBn_FrEvaluatePolynomial(result: *mut Fr, poly: *const Fr, bufSize: usize, x: *const Fr);

    // Fr
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

    // Fp
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

    // Fp2
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

    // G1
    fn mclBnG1_isEqual(x: *const G1, y: *const G1) -> i32;
    fn mclBnG1_isValid(x: *const G1) -> i32;
    fn mclBnG1_isZero(x: *const G1) -> i32;

    fn mclBnG1_setStr(x: *mut G1, buf: *const u8, bufSize: usize, ioMode: i32) -> c_int;
    fn mclBnG1_getStr(buf: *mut u8, maxBufSize: usize, x: *const G1, ioMode: i32) -> usize;
    fn mclBnG1_serialize(buf: *mut u8, maxBufSize: usize, x: *const G1) -> usize;
    fn mclBnG1_deserialize(x: *mut G1, buf: *const u8, bufSize: usize) -> usize;
    fn mclBnG1_mulVec(x: *mut G1, vec1: *const G1, vec2: *const Fr, bufSize: usize);

    fn mclBnG1_add(z: *mut G1, x: *const G1, y: *const G1);
    fn mclBnG1_sub(z: *mut G1, x: *const G1, y: *const G1);
    fn mclBnG1_neg(y: *mut G1, x: *const G1);

    fn mclBnG1_dbl(y: *mut G1, x: *const G1);
    fn mclBnG1_mul(z: *mut G1, x: *const G1, y: *const Fr);
    fn mclBnG1_normalize(y: *mut G1, x: *const G1);
    fn mclBnG1_hashAndMapTo(x: *mut G1, buf: *const u8, bufSize: usize) -> c_int;

    // G2
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

    // GT
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

pub enum CurveType {
    BN254 = 0,
    BN381 = 1,
    SNARK = 4,
    BLS12_381 = 5,
}

const MCLBN_FP_UNIT_SIZE: usize = 6;
const MCLBN_FR_UNIT_SIZE: usize = 4;
const MCLBN_COMPILED_TIME_VAR: c_int = MCLBN_FR_UNIT_SIZE as c_int * 10 + MCLBN_FP_UNIT_SIZE as c_int;

macro_rules! common_impl {
    ($t:ty, $is_equal_fn:ident, $is_zero_fn:ident) => {
        impl PartialEq for $t {
            fn eq(&self, rhs: &Self) -> bool {
                unsafe { $is_equal_fn(self, rhs) == 1 }
            }
        }
        impl $t {
            pub fn zero() -> $t {
                Default::default()
            }
            pub unsafe fn uninit() -> $t {
                std::mem::MaybeUninit::uninit().assume_init()
            }
            pub fn clear(&mut self) {
                *self = <$t>::zero()
            }
            pub fn is_zero(&self) -> bool {
                unsafe { $is_zero_fn(self) == 1 }
            }
        }
    };
}
macro_rules! is_valid_impl {
    ($t:ty, $is_valid_fn:ident) => {
        impl $t {
            pub fn is_valid(&self) -> bool {
                unsafe { $is_valid_fn(self) == 1 }
            }
        }
    };
}

macro_rules! serialize_impl {
    ($t:ty, $size:expr, $serialize_fn:ident, $deserialize_fn:ident) => {
        impl $t {
            pub fn deserialize(&mut self, buf: &[u8]) -> bool {
                unsafe { $deserialize_fn(self, buf.as_ptr(), buf.len()) > 0 }
            }
            pub fn serialize(&self) -> Vec<u8> {
                let size = unsafe { $size } as usize;
                let mut buf: Vec<u8> = Vec::with_capacity(size);
                let n: usize;
                unsafe {
                    n = $serialize_fn(buf.as_mut_ptr(), size, self);
                }
                if n == 0 {
                    panic!("serialize");
                }
                unsafe {
                    buf.set_len(n);
                }
                buf
            }
        }
    };
}

macro_rules! str_impl {
    ($t:ty, $maxBufSize:expr, $get_str_fn:ident, $set_str_fn:ident) => {
        impl $t {
            pub fn from_str(s: &str, base: i32) -> Option<$t> {
                let mut v = unsafe { <$t>::uninit() };
                if v.set_str(s, base) {
                    return Some(v);
                }
                None
            }
            pub fn set_str(&mut self, s: &str, base: i32) -> bool {
                unsafe { $set_str_fn(self, s.as_ptr(), s.len(), base) == 0 }
            }
            pub fn get_str(&self, io_mode: i32) -> String {
                let mut buf: [u8; $maxBufSize] = unsafe { MaybeUninit::uninit().assume_init() };
                let n: usize;
                unsafe {
                    n = $get_str_fn(buf.as_mut_ptr(), buf.len(), self, io_mode);
                }
                if n == 0 {
                    panic!("mclBnFr_getStr");
                }
                unsafe { std::str::from_utf8_unchecked(&buf[0..n]).into() }
            }
        }
    };
}

macro_rules! int_impl {
    ($t:ty, $set_int_fn:ident, $is_one_fn:ident) => {
        impl $t {
            pub fn from_int(x: i32) -> $t {
                let mut v = unsafe { <$t>::uninit() };
                v.set_int(x);
                v
            }
            pub fn set_int(&mut self, x: i32) {
                unsafe {
                    $set_int_fn(self, x);
                }
            }
            pub fn is_one(&self) -> bool {
                unsafe { $is_one_fn(self) == 1 }
            }
        }
    };
}

macro_rules! base_field_impl {
    ($t:ty,  $set_little_endian_fn:ident, $set_little_endian_mod_fn:ident, $set_hash_of_fn:ident, $set_by_csprng_fn:ident, $is_odd_fn:ident, $is_negative_fn:ident, $square_root_fn:ident) => {
        impl $t {
            pub fn set_little_endian(&mut self, buf: &[u8]) -> bool {
                unsafe { $set_little_endian_fn(self, buf.as_ptr(), buf.len()) == 0 }
            }
            pub fn set_little_endian_mod(&mut self, buf: &[u8]) -> bool {
                unsafe { $set_little_endian_mod_fn(self, buf.as_ptr(), buf.len()) == 0 }
            }
            pub fn set_hash_of(&mut self, buf: &[u8]) -> bool {
                unsafe { $set_hash_of_fn(self, buf.as_ptr(), buf.len()) == 0 }
            }
            pub fn set_by_csprng(&mut self) {
                unsafe { $set_by_csprng_fn(self) }
            }
            pub fn is_odd(&self) -> bool {
                unsafe { $is_odd_fn(self) == 1 }
            }
            pub fn is_negative(&self) -> bool {
                unsafe { $is_negative_fn(self) == 1 }
            }
            pub fn square_root(y: &mut $t, x: &$t) -> bool {
                unsafe { $square_root_fn(y, x) == 0 }
            }
        }
    };
}

macro_rules! add_op_impl {
    ($t:ty, $add_fn:ident, $sub_fn:ident, $neg_fn:ident) => {
        impl $t {
            pub fn add(z: &mut $t, x: &$t, y: &$t) {
                unsafe { $add_fn(z, x, y) }
            }
            pub fn sub(z: &mut $t, x: &$t, y: &$t) {
                unsafe { $sub_fn(z, x, y) }
            }
            pub fn neg(y: &mut $t, x: &$t) {
                unsafe { $neg_fn(y, x) }
            }
        }
        impl<'a> Add for &'a $t {
            type Output = $t;
            fn add(self, other: &$t) -> $t {
                let mut v = unsafe { <$t>::uninit() };
                <$t>::add(&mut v, &self, &other);
                v
            }
        }
        impl<'a> AddAssign<&'a $t> for $t {
            fn add_assign(&mut self, other: &$t) {
                // how can I write this?
                // unsafe { <$t>::add(&mut self, &self, &other); }
                let mut v = unsafe { <$t>::uninit() };
                <$t>::add(&mut v, &self, &other);
                *self = v;
            }
        }
        impl<'a> Sub for &'a $t {
            type Output = $t;
            fn sub(self, other: &$t) -> $t {
                let mut v = unsafe { <$t>::uninit() };
                <$t>::sub(&mut v, &self, &other);
                v
            }
        }
        impl<'a> SubAssign<&'a $t> for $t {
            fn sub_assign(&mut self, other: &$t) {
                let mut v = unsafe { <$t>::uninit() };
                <$t>::sub(&mut v, &self, &other);
                *self = v;
            }
        }
    };
}

macro_rules! field_mul_op_impl {
    ($t:ty, $mul_fn:ident, $div_fn:ident, $inv_fn:ident, $sqr_fn:ident) => {
        impl $t {
            pub fn mul(z: &mut $t, x: &$t, y: &$t) {
                unsafe { $mul_fn(z, x, y) }
            }
            pub fn div(z: &mut $t, x: &$t, y: &$t) {
                unsafe { $div_fn(z, x, y) }
            }
            pub fn inv(y: &mut $t, x: &$t) {
                unsafe { $inv_fn(y, x) }
            }
            pub fn sqr(y: &mut $t, x: &$t) {
                unsafe { $sqr_fn(y, x) }
            }
        }
        impl<'a> Mul for &'a $t {
            type Output = $t;
            fn mul(self, other: &$t) -> $t {
                let mut v = unsafe { <$t>::uninit() };
                <$t>::mul(&mut v, &self, &other);
                v
            }
        }
        impl<'a> MulAssign<&'a $t> for $t {
            fn mul_assign(&mut self, other: &$t) {
                let mut v = unsafe { <$t>::uninit() };
                <$t>::mul(&mut v, &self, &other);
                *self = v;
            }
        }
        impl<'a> Div for &'a $t {
            type Output = $t;
            fn div(self, other: &$t) -> $t {
                let mut v = unsafe { <$t>::uninit() };
                <$t>::div(&mut v, &self, &other);
                v
            }
        }
        impl<'a> DivAssign<&'a $t> for $t {
            fn div_assign(&mut self, other: &$t) {
                let mut v = unsafe { <$t>::uninit() };
                <$t>::div(&mut v, &self, &other);
                *self = v;
            }
        }
    };
}

macro_rules! ec_impl {
    ($t:ty, $dbl_fn:ident, $mul_fn:ident, $normalize_fn:ident, $set_hash_and_map_fn:ident) => {
        impl $t {
            pub fn dbl(y: &mut $t, x: &$t) {
                unsafe { $dbl_fn(y, x) }
            }
            pub fn mul(z: &mut $t, x: &$t, y: &Fr) {
                unsafe { $mul_fn(z, x, y) }
            }
            pub fn normalize(y: &mut $t, x: &$t) {
                unsafe { $normalize_fn(y, x) }
            }
            pub fn set_hash_of(&mut self, buf: &[u8]) -> bool {
                unsafe { $set_hash_and_map_fn(self, buf.as_ptr(), buf.len()) == 0 }
            }
        }
    };
}

#[derive(Default, Debug, Clone)]
#[repr(C)]
pub struct Fp {
    d: [u64; MCLBN_FP_UNIT_SIZE],
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
add_op_impl![Fp, mclBnFp_add, mclBnFp_sub, mclBnFp_neg];
field_mul_op_impl![Fp, mclBnFp_mul, mclBnFp_div, mclBnFp_inv, mclBnFp_sqr];

#[derive(Default, Debug, Clone)]
#[repr(C)]
pub struct Fp2 {
    d: [Fp; 2],
}
common_impl![Fp2, mclBnFp2_isEqual, mclBnFp2_isZero];
serialize_impl![
    Fp2,
    mclBn_getFpByteSize() * 2,
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

#[derive(Default, Debug, Clone)]
#[repr(C)]
pub struct Fr {
    d: [u64; MCLBN_FR_UNIT_SIZE],
}
impl Fr {
    pub fn get_order() -> String {
        get_curve_order()
    }
}
common_impl![Fr, mclBnFr_isEqual, mclBnFr_isZero];
is_valid_impl![Fr, mclBnFr_isValid];
serialize_impl![
    Fr,
    mclBn_getFrByteSize(),
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
    mclBn_getFpByteSize(),
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
    mclBn_getFpByteSize() * 2,
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

#[derive(Default, Debug, Clone)]
#[repr(C)]
pub struct GT {
    d: [Fp; 12],
}
common_impl![GT, mclBnGT_isEqual, mclBnGT_isZero];
serialize_impl![
    GT,
    mclBn_getFpByteSize() * 12,
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

pub fn get_version() -> u32 {
    unsafe { mclBn_getVersion() }
}

pub fn init(curve: CurveType) -> bool {
    unsafe { mclBn_init(curve as c_int, MCLBN_COMPILED_TIME_VAR) == 0 }
}

pub fn get_fr_serialized_size() -> u32 {
    unsafe { mclBn_getFrByteSize() as u32 }
}

pub fn get_fp_serialized_size() -> u32 {
    unsafe { mclBn_getFpByteSize() as u32 }
}

pub fn get_g1_serialized_size() -> u32 {
    get_fp_serialized_size()
}

pub fn get_g2_serialized_size() -> u32 {
    get_fp_serialized_size() * 2
}

pub fn get_gt_serialized_size() -> u32 {
    get_fp_serialized_size() * 12
}

macro_rules! get_str_impl {
    ($get_str_fn:ident) => {{
        let mut buf: [u8; 256] = unsafe { MaybeUninit::uninit().assume_init() };
        let n: usize;
        unsafe {
            n = $get_str_fn(buf.as_mut_ptr(), buf.len());
        }
        if n == 0 {
            panic!("get_str");
        }
        unsafe { std::str::from_utf8_unchecked(&buf[0..n]).into() }
    }};
}

pub fn get_field_order() -> String {
    get_str_impl![mclBn_getFieldOrder]
}

pub fn get_curve_order() -> String {
    get_str_impl![mclBn_getCurveOrder]
}

pub fn pairing(z: &mut GT, x: &G1, y: &G2) {
    unsafe {
        mclBn_pairing(z, x, y);
    }
}

pub fn miller_loop(z: &mut GT, x: &G1, y: &G2) {
    unsafe {
        mclBn_millerLoop(z, x, y);
    }
}

pub fn final_exp(y: &mut GT, x: &GT) {
    unsafe {
        mclBn_finalExp(y, x);
    }
}


// KZG 10

const G1_GEN_X: &str = "3685416753713387016781088315183077757961620795782546409894578378688607592378376318836054947676345821548104185464507";
const G1_GEN_Y: &str = "1339506544944476473020471379941921221584933875938349620426543736416511423956333506472724655353366534992391756441569";
const G2_GEN_X_D0: &str = "352701069587466618187139116011060144890029952792775240219908644239793785735715026873347600343865175952761926303160";
const G2_GEN_X_D1: &str = "3059144344244213709971259814753781636986470325476647558659373206291635324768958432433509563104347017837885763365758";
const G2_GEN_Y_D0: &str = "1985150602287291935568054521177171638300868978215655730859378665066344726373823718423869104263333984641494340347905";
const G2_GEN_Y_D1: &str = "927553665492332455747201965776037880757740193453592970025027978793976877002675564980949289727957565575433344219582";

impl G1 {
    pub fn gen() -> G1 {
        let mut g1 = G1::default();
        g1.x.set_str(G1_GEN_X, 10);
        g1.y.set_str(G1_GEN_Y, 10);
        g1.z.set_int(1);
        return g1;
    }

    pub fn pair(&self, rhs: &G2) -> GT {
        let mut gt = GT::default();

        pairing(&mut gt, &self, &rhs);

        return gt;
    }
}

impl ops::Mul<&Fr> for &G1 {
    type Output = G1;
    fn mul(self, rhs: &Fr) -> Self::Output {
        let mut g1 = G1::default();
        G1::mul(&mut g1, &self, &rhs);

        return g1;
    }
}

impl ops::Sub<G1> for G1 {
    type Output = G1;
    fn sub(self, rhs: G1) -> Self::Output {
        let mut g1 = G1::default();
        G1::sub(&mut g1, &self, &rhs);

        return g1;
    }
}

impl GT {
    pub fn get_final_exp(&self) -> GT { 
        let mut gt = GT::default();
        final_exp(&mut gt, &self);

        return gt;
    }

    pub fn get_inv(&self) -> GT {
        let mut gt = GT::default();
        GT::inv(&mut gt, self);

        return gt;
    }
}

impl ops::Mul<GT> for GT {
    type Output = GT;
    fn mul(self, rhs: GT) -> Self::Output {
        let mut gt = GT::default();
        GT::mul(&mut gt, &self, &rhs);

        return gt;
    }
}

impl G2 {
    pub fn gen() -> G2 {
        let mut g2 = G2::default();
        
        g2.x.d[0].set_str(G2_GEN_X_D0, 10);
        g2.x.d[1].set_str(G2_GEN_X_D1, 10);
        g2.y.d[0].set_str(G2_GEN_Y_D0, 10);
        g2.y.d[1].set_str(G2_GEN_Y_D1, 10);
        g2.z.d[0].set_int(1);
        g2.z.d[1].clear();

        return g2;
    }
}

impl ops::Mul<&Fr> for &G2 {
    type Output = G2;
    fn mul(self, rhs: &Fr) -> Self::Output {
        let mut g2 = G2::default();
        G2::mul(&mut g2, &self, &rhs);

        return g2;
    }
}

impl ops::Sub<G2> for G2 {
    type Output = G2;
    fn sub(self, rhs: G2) -> Self::Output {
        let mut g2 = G2::default();
        G2::sub(&mut g2, &self, &rhs);

        return g2;
    }
}

impl Fr {
    pub fn one() -> Fr {
        Fr::from_int(1)
    }

    pub fn get_neg(&self) -> Fr {
        let mut fr = Fr::default();
        Fr::neg(&mut fr, self);

        return fr;
    }

    pub fn get_inv(&self) -> Fr {
        let mut fr = Fr::default();
        Fr::inv(&mut fr, self);

        return fr;
    }

    pub fn random() -> Fr {
        let mut fr = Fr::default();
        Fr::set_by_csprng(&mut fr);

        return fr;
    }
}

impl ops::Mul<Fr> for Fr {
    type Output = Fr;
    fn mul(self, rhs: Fr) -> Self::Output {
        let mut result = Fr::default();
        Fr::mul(&mut result, &self, &rhs);

        return result;
    }
}

impl ops::Div<Fr> for Fr {
    type Output = Fr;
    fn div(self, rhs: Fr) -> Self::Output {
        let mut result = Fr::default();
        Fr::div(&mut result, &self, &rhs);

        return result;
    }
}

impl ops::Sub<Fr> for Fr {
    type Output = Fr;
    fn sub(self, rhs: Fr) -> Self::Output {
        let mut result = Fr::default();
        Fr::sub(&mut result, &self, &rhs);

        return result;
    }
}

// KZG 10 Impl

#[derive(Debug, Clone)]
pub struct Polynomial {
    pub coeffs: Vec<Fr>
}

#[derive(Debug, Clone)]
pub struct Curve {
    pub g1_gen: G1,
    pub g2_gen: G2,
    pub g1_points: Vec<G1>,
    pub g2_points: Vec<G2>,
    pub order: usize
}

impl Polynomial {

    pub fn from_fr(data: Vec<Fr>) -> Self {
        Self {
            coeffs: data
        }
    }
    
    pub fn from_i32(data: &Vec<i32>) -> Self {
        Self {
            coeffs: data.iter().map(|x| Fr::from_int(*x)).collect(),
        }
    }

    pub fn order(&self) -> usize {
        self.coeffs.len()
    }

    pub fn eval_at(&self, point: &Fr) -> Fr {
        let mut result = Fr::default();
        unsafe { 
            mclBn_FrEvaluatePolynomial(&mut result, self.coeffs.as_ptr(), self.order(), point)
        };
        return result;
    }

    pub fn gen_proof_at(&self, g1_points: &Vec<G1>, point: &Fr) -> G1 {
        let divisor = vec![point.get_neg(), Fr::one()];
        let quotient_poly = self.long_division(&divisor);

        let mut result = G1::default();
        unsafe {
            mclBnG1_mulVec(&mut result, g1_points.as_ptr(), quotient_poly.coeffs.as_ptr(), min(g1_points.len(), quotient_poly.order()))
        };
        return result;
    }

    pub fn long_division(&self, divisor: &Vec<Fr>) -> Polynomial {
        let mut poly_copy = self.clone();
        let mut copy_pos = poly_copy.order() - 1;

        let mut result = vec![Fr::default(); poly_copy.order() - divisor.len() + 1];
        
        for r_i in (0 .. result.len()).rev() {
            result[r_i] = &poly_copy.coeffs[copy_pos] / &divisor.last().unwrap();

            for d_i in (0 .. divisor.len()).rev() {
                poly_copy.coeffs[r_i + d_i] -= &(&result[r_i] * &divisor[d_i]);
            }

            copy_pos -= 1;
        }

        return Polynomial {
            coeffs: result
        };
    }

    pub fn commit(& self, g1_points: &Vec<G1>) -> G1 {
        let mut result = G1::default();
        unsafe {
            mclBnG1_mulVec(&mut result, g1_points.as_ptr(), self.coeffs.as_ptr(), min(g1_points.len(), self.order()))
        };
        return result;
    }

    pub fn random(order: usize) -> Polynomial {
        let coeffs = iter::repeat(0)
            .take(order)
            .map(|_| Fr::random())
            .collect();

        return Polynomial {
            coeffs
        };
    }
}

impl Curve {
    pub fn new(secret: &Fr, order: usize) -> Self {
        let g1_gen = G1::gen();
        let g2_gen = G2::gen(); 

        let mut g1_points = vec!(G1::default(); order);
        let mut g2_points = vec!(G2::default(); order);

        let mut secret_to_power = Fr::one();
        for i in 0..order {
            G1::mul(&mut (g1_points[i]), &g1_gen, &secret_to_power);
            G2::mul(&mut (g2_points[i]), &g2_gen, &secret_to_power);

            secret_to_power *= &secret;
        }

        Self {
            g1_gen,
            g2_gen,
            g1_points,
            g2_points,
            order
        }
    }

    pub fn is_proof_valid(&self, commitment: &G1, proof: &G1, x: &Fr, y: &Fr) -> bool {
        let secret_minus_x = &self.g2_points[1] - &(&self.g2_gen * x); // g2 * x to get x on g2
        let commitment_minus_y = commitment - &(&self.g1_gen * y);

        return self.verify_pairing(&commitment_minus_y, &self.g2_gen, proof, &secret_minus_x);
    }

    pub fn verify_pairing(&self, a1: &G1, a2: &G2, b1: &G1, b2: &G2) -> bool {
        let pairing1 = a1.pair(&a2).get_inv();
        let pairing2 = b1.pair(&b2);

        let result = (pairing1 * pairing2).get_final_exp();

        return result.is_one();
    }
}


// FK20 | FFT

pub static mut SCALE_2_ROOT_OF_UNITY: Vec<Fr> = vec![];
pub static mut GLOBALS_INITIALIZED: bool = false;
pub const PRIMITIVE_ROOT: i32 = 5;

pub unsafe fn init_globals() {
    if GLOBALS_INITIALIZED {
        return;
    }
    // MODULUS = 52435875175126190479447740508185965837690552500527637822603658699938581184513
	// PRIMITIVE_ROOT = 5
	// [pow(PRIMITIVE_ROOT, (MODULUS - 1) // (2**i), MODULUS) for i in range(32)]
    // TODO: gen dynamically?
    SCALE_2_ROOT_OF_UNITY = vec![
		/* k=0          r=1          */ "1",
		/* k=1          r=2          */ "52435875175126190479447740508185965837690552500527637822603658699938581184512",
		/* k=2          r=4          */ "3465144826073652318776269530687742778270252468765361963008",
		/* k=3          r=8          */ "28761180743467419819834788392525162889723178799021384024940474588120723734663",
		/* k=4          r=16         */ "35811073542294463015946892559272836998938171743018714161809767624935956676211",
		/* k=5          r=32         */ "32311457133713125762627935188100354218453688428796477340173861531654182464166",
		/* k=6          r=64         */ "6460039226971164073848821215333189185736442942708452192605981749202491651199",
		/* k=7          r=128        */ "3535074550574477753284711575859241084625659976293648650204577841347885064712",
		/* k=8          r=256        */ "21071158244812412064791010377580296085971058123779034548857891862303448703672",
		/* k=9          r=512        */ "12531186154666751577774347439625638674013361494693625348921624593362229945844",
		/* k=10         r=1024       */ "21328829733576761151404230261968752855781179864716879432436835449516750606329",
		/* k=11         r=2048       */ "30450688096165933124094588052280452792793350252342406284806180166247113753719",
		/* k=12         r=4096       */ "7712148129911606624315688729500842900222944762233088101895611600385646063109",
		/* k=13         r=8192       */ "4862464726302065505506688039068558711848980475932963135959468859464391638674",
		/* k=14         r=16384      */ "36362449573598723777784795308133589731870287401357111047147227126550012376068",
		/* k=15         r=32768      */ "30195699792882346185164345110260439085017223719129789169349923251189180189908",
		/* k=16         r=65536      */ "46605497109352149548364111935960392432509601054990529243781317021485154656122",
		/* k=17         r=131072     */ "2655041105015028463885489289298747241391034429256407017976816639065944350782",
		/* k=18         r=262144     */ "42951892408294048319804799042074961265671975460177021439280319919049700054024",
		/* k=19         r=524288     */ "26418991338149459552592774439099778547711964145195139895155358980955972635668",
		/* k=20         r=1048576    */ "23615957371642610195417524132420957372617874794160903688435201581369949179370",
		/* k=21         r=2097152    */ "50175287592170768174834711592572954584642344504509533259061679462536255873767",
		/* k=22         r=4194304    */ "1664636601308506509114953536181560970565082534259883289958489163769791010513",
		/* k=23         r=8388608    */ "36760611456605667464829527713580332378026420759024973496498144810075444759800",
		/* k=24         r=16777216   */ "13205172441828670567663721566567600707419662718089030114959677511969243860524",
		/* k=25         r=33554432   */ "10335750295308996628517187959952958185340736185617535179904464397821611796715",
		/* k=26         r=67108864   */ "51191008403851428225654722580004101559877486754971092640244441973868858562750",
		/* k=27         r=134217728  */ "24000695595003793337811426892222725080715952703482855734008731462871475089715",
		/* k=28         r=268435456  */ "18727201054581607001749469507512963489976863652151448843860599973148080906836",
		/* k=29         r=536870912  */ "50819341139666003587274541409207395600071402220052213520254526953892511091577",
		/* k=30         r=1073741824 */ "3811138593988695298394477416060533432572377403639180677141944665584601642504",
		/* k=31         r=2147483648 */ "43599901455287962219281063402626541872197057165786841304067502694013639882090",
    ].into_iter()
    .map(|x| Fr::from_str(x, 10).unwrap())
    .collect();

    GLOBALS_INITIALIZED = true;
}

pub fn expand_root_of_unity(root: &Fr) -> Vec<Fr> {
    let mut root_z = vec![Fr::one(), root.clone()];
    let mut i = 1;
    while !root_z[i].is_one() {
        let next = &root_z[i] * &root;
        root_z.push(next);
        i += 1;
    }
    return root_z;
}

pub struct FFTSettings {
    pub max_width: usize,
    pub root_of_unity: Fr,
    pub exp_roots_of_unity: Vec<Fr>,
    pub exp_roots_of_unity_rev: Vec<Fr>
}

impl FFTSettings {
    pub fn new(max_scale: u8) -> FFTSettings {
        let root: Fr;
        unsafe {
            init_globals();
            root = SCALE_2_ROOT_OF_UNITY[max_scale as usize].clone()
        }
        let root_z = expand_root_of_unity(&root);
        let mut root_z_rev = root_z.clone();
        root_z_rev.reverse();

        FFTSettings {
            max_width: 1 << max_scale,
            root_of_unity: root,
            exp_roots_of_unity: root_z,
            exp_roots_of_unity_rev: root_z_rev
        }
    }

    fn _fft(&self, values: &[Fr], offset: usize, stride: usize, roots_of_unity: &Vec<Fr>, root_stride: usize, out: &mut [Fr]) {
        // check if correct value is checked in case of a bug!
        if out.len() <= 4 { // if the value count is small, run the unoptimized version instead. // TODO tune threshold.
            return self._simple_ftt(values, offset, stride, roots_of_unity, root_stride, out);
        }

        let half = out.len() >> 1;

        // left
        self._fft(values, offset, stride << 1, roots_of_unity, root_stride << 1, &mut out[..half]);
        // right
        self._fft(values, offset + stride, stride << 1, roots_of_unity, root_stride << 1, &mut out[half..]);

        for i in 0..half {
            let x = out[i].clone();
            let y = out[i + half].clone();
            let root = &roots_of_unity[i * root_stride];

            let y_times_root = &y * root;
            out[i] = &x + &y_times_root;
            out[i + half] = &x - &y_times_root;
        }
    }

    fn _simple_ftt(&self, values: &[Fr], offset: usize, stride: usize, roots_of_unity: &Vec<Fr>, root_stride: usize, out: &mut [Fr]) {
        let out_len = out.len();
        let init_last = &values[offset] * &roots_of_unity[0];

        for i in 0..out_len {
            let mut last = init_last.clone();
            for j in 1..out_len {
                let jv = &values[offset + j * stride];
                let r = &roots_of_unity[((i * j) % out_len) * root_stride];
                // last += (jv * r)
                last = &last.clone() + &(jv * r);
            }
            out[i] = last;
        }
    }

    pub fn inplace_fft(&self, values: &[Fr], inv: bool) -> Vec<Fr> {
        
        if inv {
            let root_z: Vec<Fr> = self.exp_roots_of_unity_rev.iter().map(|x| x.clone()).take(self.max_width).collect();
            let stride = self.max_width / values.len();

            let mut out = vec![Fr::default(); values.len()];
            self._fft(&values, 0, 1, &root_z, stride, &mut out);

            let inv_len = Fr::from_int(values.len() as i32).get_inv();
            for i in 0..out.len() {
                out[i] = &out[i].clone() * &inv_len;
            }
            return out;
        } else {
            let root_z: Vec<Fr> = self.exp_roots_of_unity.iter().map(|x| x.clone()).take(self.max_width).collect();
            let stride = self.max_width / values.len();

            let mut out = vec![Fr::default(); values.len()];
            self._fft(&values, 0, 1, &root_z, stride, &mut out);

            return out;
        }
    }

    pub fn fft(&self, values: &Vec<Fr>, inv: bool) -> Vec<Fr> {
        let n = next_pow_of_2(values.len());
        
        let diff = n - values.len();
        let tail= iter::repeat(Fr::zero()).take(diff);
        let values_copy: Vec<Fr> = values.iter()
            .map(|x| x.clone())
            .chain(tail)
            .collect();

        return self.inplace_fft(&values_copy, inv);
    }

    pub fn fft_g1(&self, values: &Vec<G1>) -> Vec<G1> {
        // TODO: check if copy can be removed, opt?
        let vals_copy = values.clone();
        
        let root_z: Vec<Fr> = self.exp_roots_of_unity.iter()
            .take(self.max_width)
            .map(|x| x.clone())
            .collect();

        let stride = self.max_width /  values.len();
        let mut out = vec![G1::zero(); values.len()];

        FFTSettings::_fft_g1(&self, &vals_copy, 0, 1, &root_z, stride, &mut out);

        return out;
    }

    fn _fft_g1(fft_settings: &FFTSettings, values: &Vec<G1>, value_offset: usize, value_stride: usize, roots_of_unity: &Vec<Fr>, roots_stride: usize, out: &mut [G1]) {
        //TODO: fine tune for opt, maybe resolve number dinamically based on experiments
        if out.len() <= 4 {
            return FFTSettings::_fft_g1_simple(values, value_offset, value_stride, roots_of_unity, roots_stride, out);
        }

        let half = out.len() >> 1;

        // left
        FFTSettings::_fft_g1(fft_settings, values, value_offset, value_stride << 1, roots_of_unity, roots_stride << 1, &mut out[..half]);
        // right
        FFTSettings::_fft_g1(fft_settings, values, value_offset + value_stride, value_stride << 1, roots_of_unity, roots_stride << 1, &mut out[half..]);

        for i in 0..half {
            let x = out[i].clone();
            let y = out[i + half].clone();
            let root = &roots_of_unity[i * roots_stride];

            let y_times_root = &y * &root;
            out[i] = &x + &y_times_root;
            out[i + half] = &x - &y_times_root;
        }

        return;
    }
    

    fn _fft_g1_simple(values: &Vec<G1>, value_offset: usize, value_stride: usize, roots_of_unity: &Vec<Fr>, roots_stride: usize, out: &mut [G1]) {
        let l = out.len();
        for i in 0..l {
            // TODO: check this logic with a working brain, there could be a simpler way to write this;
            let mut v = &values[value_offset] * &roots_of_unity[0];
            let mut last = v.clone();
            for j in 1..l {
                v = &values[value_offset + j * value_stride] * &roots_of_unity[((i * j) % l) * roots_stride];
                let temp = last.clone();
                last = &temp + &v;
            }
            out[i] = last;
        }
    }
}

// KZG Settings + FK20 Settings + FFTSettings?
pub struct FK20Matrix {
    pub curve: Curve,
    pub x_ext_fft_files: Vec<Vec<G1>>,
    pub fft_settings: FFTSettings,
    pub chunk_len: usize,
}

impl FK20Matrix {
    
    pub fn new(curve: Curve, n2: usize, chunk_len: usize, fft_max_scale: u8) -> FK20Matrix {
        let n = n2 >> 1; // div by 2
        let k = n / chunk_len;
        let fft_settings = FFTSettings::new(fft_max_scale);
        if n2 > fft_settings.max_width {
            panic!("extended size is larger than fft settings supoort");
        }
        // TODO: more panic checks
        
        let mut x_ext_fft_files: Vec<Vec<G1>> = vec![vec![]; chunk_len];
        for i in 0..chunk_len {
            x_ext_fft_files[i] = FK20Matrix::x_ext_fft_precompute(&fft_settings, &curve, n, k, chunk_len,i);
        }

        FK20Matrix {
            curve,
            x_ext_fft_files,
            fft_settings,
            chunk_len
        }
    }
    
    fn x_ext_fft_precompute(fft_settings: &FFTSettings, curve: &Curve, n: usize, k: usize, chunk_len: usize, offset: usize) -> Vec<G1> {
        let mut x: Vec<G1> = vec![G1::default(); k];
        let start = n - chunk_len - offset - 1;

        let mut i = 0;
        let mut j = start + chunk_len;

        while i + 1 < k {
            // hack to remove overflow checking, 
            // could just move this to the bottom and define j as start, but then need to check for overflows
            // basically last j -= chunk_len overflows, but it's not used to access the array, as the i + 1 < k is false
            j -= chunk_len;
            x[i] = curve.g1_points[j].clone();
            i += 1;
        }
        
        x[k - 1] = G1::zero();

        return FK20Matrix::toeplitz_part_1(&fft_settings, &x);
    }

    pub fn toeplitz_part_1(fft_settings: &FFTSettings, x: &Vec<G1>) -> Vec<G1> {
        let n = x.len();

        // extend x with zeroes
        let tail= vec![G1::zero(); n];
        let x_ext: Vec<G1> = x.iter()
            .map(|g1| g1.clone())
            .chain(tail)
            .collect();

        let x_ext_fft = FK20Matrix::fft_g1(&fft_settings, &x_ext);
        
        return x_ext_fft;
    }

    pub fn fft_g1(fft_settings: &FFTSettings, values: &Vec<G1>) -> Vec<G1> {
        // TODO: check if copy can be removed, opt?
        let vals_copy = values.clone();
        
        let root_z: Vec<Fr> = fft_settings.exp_roots_of_unity.iter()
            .take(fft_settings.max_width)
            .map(|x| x.clone())
            .collect();

        let stride = fft_settings.max_width /  values.len();
        let mut out = vec![G1::zero(); values.len()];

        FK20Matrix::_fft_g1(&fft_settings, &vals_copy, 0, 1, &root_z, stride, &mut out);

        return out;
    }

 
    pub fn fft_g1_inv(&self, values: &Vec<G1>) -> Vec<G1> {
        // TODO: check if copy can be removed, opt?
        let vals_copy = values.clone();
        
        let root_z: Vec<Fr> = self.fft_settings.exp_roots_of_unity_rev.iter()
            .take(self.fft_settings.max_width)
            .map(|x| x.clone())
            .collect();

        let stride = self.fft_settings.max_width /  values.len();
        let mut out = vec![G1::zero(); values.len()];

        FK20Matrix::_fft_g1(&self.fft_settings, &vals_copy, 0, 1, &root_z, stride, &mut out);
        
        let inv_len = Fr::from_int(values.len() as i32).get_inv();
        for i in 0..out.len() {
            let tmp = &out[i] * &inv_len;
            out[i] = tmp;
        }

        return out;
    }

    pub fn dau_using_fk20_multi(&self, polynomial: &Polynomial) -> Vec<G1> {
        let n = polynomial.order();
        //TODO: checks? -> perfmance hit tho?
        let n2 = n << 1;
        let extended_poly = polynomial.get_extended(n2);

        let mut proofs = extended_poly.fk20_multi_dao_optimized(&self);

        order_by_rev_bit_order(&mut proofs);

        return proofs;
    }

    fn _fft_g1(fft_settings: &FFTSettings, values: &Vec<G1>, value_offset: usize, value_stride: usize, roots_of_unity: &Vec<Fr>, roots_stride: usize, out: &mut [G1]) {
        //TODO: fine tune for opt, maybe resolve number dinamically based on experiments
        if out.len() <= 4 {
            return FK20Matrix::_fft_g1_simple(values, value_offset, value_stride, roots_of_unity, roots_stride, out);
        }

        let half = out.len() >> 1;

        // left
        FK20Matrix::_fft_g1(fft_settings, values, value_offset, value_stride << 1, roots_of_unity, roots_stride << 1, &mut out[..half]);
        // right
        FK20Matrix::_fft_g1(fft_settings, values, value_offset + value_stride, value_stride << 1, roots_of_unity, roots_stride << 1, &mut out[half..]);

        for i in 0..half {
            let x = out[i].clone();
            let y = out[i + half].clone();
            let root = &roots_of_unity[i * roots_stride];

            let y_times_root = &y * &root;
            out[i] = &x + &y_times_root;
            out[i + half] = &x - &y_times_root;
        }

        return;
    }
    

    fn _fft_g1_simple(values: &Vec<G1>, value_offset: usize, value_stride: usize, roots_of_unity: &Vec<Fr>, roots_stride: usize, out: &mut [G1]) {
        let l = out.len();
        for i in 0..l {
            // TODO: check this logic with a working brain, there could be a simpler way to write this;
            let mut v = &values[value_offset] * &roots_of_unity[0];
            let mut last = v.clone();
            for j in 1..l {
                v = &values[value_offset + j * value_stride] * &roots_of_unity[((i * j) % l) * roots_stride];
                let temp = last.clone();
                last = &temp + &v;
            }
            out[i] = last;
        }
    }

    fn toeplitz_coeffs_step_strided(&self, poly: &Vec<Fr>, offset: usize) -> Vec<Fr> {
        let stride = self.chunk_len;
        let n = poly.len();
        let k = n / stride;
        let k2 = k << 1;

        // [last] + [0]*(n+1) + [1 .. n-2]
        let mut toeplitz_coeffs = vec![Fr::zero(); k2];
        toeplitz_coeffs[0] = poly[n - 1 - offset].clone();
        
        let mut j = (stride << 1) - offset - 1;
        for i in k+2..k2 {
            toeplitz_coeffs[i] = poly[j].clone();
            j += stride;
        }

        return toeplitz_coeffs;
    }

    pub fn toeplitz_part_2(&self, coeffs: &Vec<Fr>, index: usize) -> Vec<G1> {
        let toeplitz_coeffs_fft = self.fft_settings.fft(&coeffs, false);

        let x_ext_fft = &self.x_ext_fft_files[index];

        let h_ext_fft: Vec<G1> = x_ext_fft.iter()
            .zip(toeplitz_coeffs_fft)
            .map(|(g1, coeff)| g1 * &coeff)
            .collect();

        return h_ext_fft;
    }

    // TODO: optimization, reuse h_ext_fft
    pub fn toeplitz_part_3(&self, h_ext_fft: &Vec<G1>) -> Vec<G1> {
        let out = self.fft_g1_inv(&h_ext_fft);

        // return half, can just resize the vector to be half.
        return out.iter().take(out.len() >> 1).map(|x| x.clone()).collect();
    }

    pub fn check_proof_multi(&self, commitment: &G1, proof: &G1, x: &Fr, ys: &Vec<Fr>) -> bool {
        let mut interpolation_poly = self.fft_settings.fft(&ys, true);
        let mut x_pow = Fr::one();
        for i in 0.. interpolation_poly.len() {
            interpolation_poly[i] *= &x_pow.get_inv();
            x_pow *= x;
        }

        let xn2 = &self.curve.g2_gen * &x_pow;
        let xn_minus_yn = &self.curve.g2_points[ys.len()] - &xn2;

        let mut result = G1::default();
        unsafe {
            mclBnG1_mulVec(&mut result, self.curve.g1_points.as_ptr(), interpolation_poly.as_ptr(), interpolation_poly.len())
        };

        let commit_minus_interp = commitment - &result;

        return self.curve.verify_pairing(&commit_minus_interp, &self.curve.g2_gen, &proof, &&xn_minus_yn);
    }
}

impl Polynomial {
    pub fn extend(vec: &Vec<Fr>, size: usize) -> Vec<Fr> {
        let to_pad = size - vec.len();
        let tail = iter::repeat(Fr::zero()).take(to_pad);
        let result: Vec<Fr> = vec.iter().map(|x| x.clone()).chain(tail).collect();

        return result;
    }

    pub fn get_extended(&self, size: usize) -> Polynomial { 
        return Polynomial::from_fr(Polynomial::extend(&self.coeffs, size));
    }

    pub fn fk20_multi_dao_optimized(&self, matrix: &FK20Matrix) -> Vec<G1> {
        let n = self.order() >> 1;
        let k = n / matrix.chunk_len;
        let k2 = k << 1;
        
        let mut h_ext_fft = vec![G1::zero(); k2];
        // TODO: this operates on an extended poly, but doesn't use the extended values?
        // literally just using the poly without the zero trailing tail, makes more sense to take it in as a param, or use without the tail;
        let reduced_poly: Vec<Fr> = self.coeffs.iter().map(|x| x.clone()).take(n).collect();

        for i in 0..matrix.chunk_len {
            let toeplitz_coeffs = matrix.toeplitz_coeffs_step_strided(&reduced_poly, i);
            let h_ext_fft_file = matrix.toeplitz_part_2(&toeplitz_coeffs, i);

            for j in 0..k2 {
                let tmp = &h_ext_fft[j] + &h_ext_fft_file[j];
                h_ext_fft[j] = tmp;
            }
        }
        
        let tail = iter::repeat(G1::zero()).take(k);
        let h: Vec<G1> = matrix.toeplitz_part_3(&h_ext_fft)
            .into_iter()
            .take(k)
            .chain(tail)
            .collect();
        
        return FK20Matrix::fft_g1(&matrix.fft_settings, &h);
    }
}

// DAS
impl FFTSettings {
    pub fn das_fft_extension(&self, values: &mut Vec<Fr>) {
        if (values.len() << 1) > self.max_width {
            panic!("ftt_settings max width too small!");
        }

        self._das_fft_extension(values, 1);
        
        // just dividing every value by 1/(2**depth) aka length
        // TODO: what's faster, maybe vec[x] * vec[x], ask herumi to implement?
        let inv_length = Fr::from_int(values.len() as i32).get_inv();
        for i in 0..values.len() {
            values[i] *= &inv_length;
        }
    }

    fn _das_fft_extension(&self, values: &mut [Fr], stride: usize) {
        if values.len() == 2 {
            let (x, y) = FFTSettings::_calc_add_and_sub(&values[0], &values[1]);

            let temp = &y * &self.exp_roots_of_unity[stride];
            values[0] = &x + &temp;
            values[1] = &x - &temp;
            return;
        }

        let length = values.len();
        let half = length >> 1;
        
        // let ab_half_0s = ab[..quarter];
        // let ab_half_1s = ab[quarter..];
        for i in 0..half {
            let (add, sub) = FFTSettings::_calc_add_and_sub(&values[i], &values[half + i]);
            values[half + i] = &sub * &self.exp_roots_of_unity_rev[(i << 1) * stride];
            values[i] = add;
        }

        // left
        self._das_fft_extension(&mut values[..half], stride << 1);
        // right
        self._das_fft_extension(&mut values[half..], stride << 1);

        for i in 0..half {
            let root = &self.exp_roots_of_unity[((i << 1) + 1) * stride];
            let y_times_root = &values[half + i] * root;

            let (add, sub) = FFTSettings::_calc_add_and_sub(&values[i], &y_times_root);
            values[i] = add;
            values[i + half] = sub;
        }
    }

    fn _calc_add_and_sub(a: &Fr, b: &Fr) -> (Fr, Fr) {
        return (a + b, a - b);
    }
}

// Data recovery

impl Polynomial {
    pub fn shift_in_place(&mut self) {
        self._shift_in_place(&Fr::from_int(PRIMITIVE_ROOT));
    }

    pub fn unshift_in_place(&mut self) {
        self._shift_in_place(&Fr::from_int(PRIMITIVE_ROOT).get_inv());
    }

    //TODO, use precalculated tables for factors?
    fn _shift_in_place(&mut self, factor: &Fr){
        let mut factor_to_power = Fr::one();
        for i in 0..self.order() {
            self.coeffs[i] *= &factor_to_power;
            factor_to_power *= factor;
        }
    }

    pub fn recover_from_samples(fft_settings: FFTSettings, samples: &[Option<Fr>]) -> Polynomial {
        let missing_data_indices: Vec<usize> = samples.iter()
            .enumerate()
            .filter(|(_, ex)| ex.is_none())
            .map(|(ix, _)| ix)
            .collect();

        let (zero_eval, zero_poly_coeffs) = fft_settings.zero_poly_via_multiplication(&missing_data_indices, samples.len());

        // TODO: possible optimization, remove clone()
        let poly_evals_with_zero: Vec<Fr> = samples.iter()
            .zip(zero_eval)
            .map(|(x, eval)| {
                if x.is_none() {
                    return Fr::zero();
                }
                return &x.clone().unwrap() * &eval;
            }).collect();

        // for val in poly_evals_with_zero {
        //     println!("{}", val.get_str(10));
        // }

        let poly_with_zero_coeffs = fft_settings.fft(&poly_evals_with_zero, true);
        let mut poly_with_zero = Polynomial::from_fr(poly_with_zero_coeffs);
        poly_with_zero.shift_in_place();

        let mut zero_poly = Polynomial::from_fr(zero_poly_coeffs);
        zero_poly.shift_in_place();

        let eval_shifted_poly_with_zero = fft_settings.fft(&poly_with_zero.coeffs, false);
        let eval_shifted_zero_poly = fft_settings.fft(&zero_poly.coeffs, false);
        
    
        let eval_shifted_reconstructed_poly: Vec<Fr> = eval_shifted_poly_with_zero.iter()
            .zip(eval_shifted_zero_poly)
            .map(|(a, b)| a / &b)
            .collect();

        let shifted_reconstructed_poly_coeffs = fft_settings.fft(&eval_shifted_reconstructed_poly, true);
        let mut shifted_reconstructed_poly = Polynomial::from_fr(shifted_reconstructed_poly_coeffs);
        shifted_reconstructed_poly.unshift_in_place();

        let reconstructed_data = fft_settings.fft(&shifted_reconstructed_poly.coeffs, false);
        
        return Polynomial::from_fr(reconstructed_data);
    }

    pub fn unwrap_default(values: &Vec<Option<Fr>>) -> Vec<Fr> {
        return values.iter().map(|x| {
            if x.is_none() {
                return Fr::zero()
            }
            return x.clone().unwrap();
        }).collect();
    }
}

// Zero Poly

impl FFTSettings {
    // TODO: could be optimized by using mutable slices!
    pub fn zero_poly_via_multiplication(&self, indices: &[usize], length: usize) -> (Vec<Fr>, Vec<Fr>) {
        if indices.is_empty() {
            return (vec![Fr::zero(); length], vec![Fr::zero(); length]);
        }

        let stride = self.max_width / length;
        let per_leaf_poly = 64;
        let per_leaf = per_leaf_poly - 1;
        if indices.len() <= per_leaf {
            let mut zero_poly = vec![Fr::default(); length];
            self.make_zero_poly_mul_leaf(&mut zero_poly, indices, stride);

            let zero_eval = self.fft(&zero_poly, false);
            return (zero_eval, zero_poly);
        }

        let leaf_count = (indices.len() + per_leaf - 1) / per_leaf;
        let n = next_pow_of_2(leaf_count * per_leaf_poly);

        // TODO: rust limitation, can't have multiple mutators for same value, code fails somewhere here, as I tried to achieve same func through duplicated value management.
        let mut out = vec![Fr::default(); n];
        let mut offset = 0;
        let mut leaves: Vec<Vec<Fr>> = vec![vec![]; leaf_count];
        let max = indices.len();
        for _ in 0..leaf_count {
            let end = min(offset + per_leaf, max);
            let mut slice = vec![Fr::default(); per_leaf_poly];
            self.make_zero_poly_mul_leaf(&mut slice, &indices[offset..end], stride);
            let mut slice_copy = slice.clone();
            out.append(&mut slice_copy);
            leaves.push(slice);
            offset += per_leaf;
        }

        let reduction_factor = 4;
        let mut scratch = vec![Fr::default(); n * 3];

        while leaves.len() > 1 {
            let reduced_count = (leaves.len() + reduction_factor - 1) / reduction_factor;
            let leaf_size = next_pow_of_2(leaves[0].len());
            for i in 0..reduced_count {
                let start = i * reduction_factor;
                let mut end = start + reduction_factor;

                let out_end = min(out.len(), end * leaf_size);
                let reduced = &mut out[start * leaf_size .. out_end].to_vec();
                end = min(end, leaves.len()); 

                let leaves_slice = &mut leaves[start..end];
                if end > start + 1 {
                    *reduced = self.reduce_leaves(&mut scratch, leaves_slice, reduced.len());
                }
                leaves[i] = reduced.to_vec();
            }
            leaves = leaves[..reduced_count].to_vec();
        }
        let zero_poly = Polynomial::extend(&leaves[0], length);
        let zero_eval = self.fft(&zero_poly, false);

        return (zero_eval, zero_poly);
    }

    pub fn reduce_leaves(&self, scratch: &mut [Fr], ps: &[Vec<Fr>], n: usize) -> Vec<Fr> {
        let out_degree: usize = ps.iter()
            .map(|x| {
                if x.len() == 0 { 
                    return 0; 
                } 
                return x.len() - 1;
            }).sum();
        let (p_padded, rest) = scratch.split_at_mut(n);
        let (mul_eval_ps, p_eval) = rest.split_at_mut(n);

        for i in 0..p_padded.len() {
            p_padded[i] = Fr::zero();
        }
        for (i, v) in ps.last().unwrap().iter().enumerate() {
            p_padded[i] = v.clone();
        }

        //can optimize this, one alloc instead of three
        let temp = self.inplace_fft(&p_padded, false);
        for i in 0..n {
            mul_eval_ps[i] = temp[i].clone();
        }

        let last_index = ps.len() - 1;
        for i in 0..last_index {
            let p = &ps[i];
            for j in 0..p.len() {
                p_padded[j] = p[j].clone();
            }
            // p_eval = inplace_fft(p_padded);
            let p_eval_result = self.inplace_fft(&p_padded, false);
            for j in 0..n {
                p_eval[j] = p_eval_result[j].clone();
            }

            for j in 0..n {
                mul_eval_ps[j] *= &p_eval[j];
            }
        }

        let result = self.inplace_fft(&mul_eval_ps, true);
        return result[..out_degree + 1].to_vec();
    }
    
    pub fn make_zero_poly_mul_leaf(&self, dest: &mut Vec<Fr>, indices: &[usize], stride: usize) {
        if (indices.len() + 1) > dest.len() {
            panic!("expected bigger dest length");
        }
        // is this neccessary?
        for i in (indices.len() + 1)..dest.len() {
            dest[i] = Fr::zero();
        }

        dest[indices.len()] = Fr::one();
        
        for (i, v) in indices.iter().enumerate() {
            let neg_di = self.exp_roots_of_unity[v * stride].get_neg();
            dest[i] = neg_di.clone();
            if i > 0 {
                let temp = &dest[i] + &dest[i - 1];
                dest[i] = temp;
                for j in (1..i).rev() {
                    dest[j] *= &neg_di;
                    let temp = &dest[j] + &dest[j - 1];
                    dest[j] = temp;
                }
                dest[0] *= &neg_di;
            }
        }
    }
}

// Misc
pub fn order_by_rev_bit_order<T>(vals: &mut Vec<T>) where T : Clone {
    let unused_bit_len = vals.len().leading_zeros() + 1;
     for i in 0..vals.len() {
        let r = i.reverse_bits() >> unused_bit_len;
        if r > i {
            let tmp = vals[r].clone();
            vals[r] = vals[i].clone();
            vals[i] = tmp;
        }
     }
}

pub fn is_power_of_2(n: usize) -> bool {
    return n & (n - 1) == 0;
}

const fn num_bits<T>() -> usize { std::mem::size_of::<T>() * 8 }

pub fn log_2(x: usize) -> usize {
    assert!(x > 0);
    num_bits::<usize>() as usize - (x.leading_zeros() as usize) - 1
}

pub fn next_pow_of_2(x: usize) -> usize {
    if x == 0 {
        return 1;
    }
    if is_power_of_2(x) {
        return x;
    }
    return 1 << (log_2(x) + 1);
}

pub fn reverse_bits_limited(length: usize, value: usize) -> usize {
    let unused_bits = length.leading_zeros();
    return value.reverse_bits() >> unused_bits;
}