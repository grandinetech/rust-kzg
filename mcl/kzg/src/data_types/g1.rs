use crate::data_types::fp::Fp;
use crate::data_types::fr::Fr;
use crate::mcl_methods;
use crate::utilities::arr64_6_to_g1_sum;
#[cfg(feature = "parallel")]
use kzg::G1 as _;
use std::ops::{Add, AddAssign};
use std::ops::{Sub, SubAssign};
use std::os::raw::c_int;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

#[link(name = "mcl", kind = "static")]
#[link(name = "mclbn384_256", kind = "static")]
#[link(name = "stdc++")]
#[allow(non_snake_case)]
extern "C" {
    pub fn mclBnG1_isEqual(x: *const G1, y: *const G1) -> i32;
    pub fn mclBnG1_isValid(x: *const G1) -> i32;
    pub fn mclBnG1_isZero(x: *const G1) -> i32;
    pub fn mclBnG1_isValidOrder(x: *const G1) -> i32;

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

pub fn g1_linear_combination(out: &mut G1, points: &[G1], scalars: &[Fr], len: usize) {
    #[cfg(feature = "parallel")]
    {
        *out = (0..len)
            .into_par_iter()
            .map(|i| points[i] * &scalars[i])
            .reduce(G1::default, |mut acc, tmp| acc.add_or_dbl(&tmp));
    }

    #[cfg(not(feature = "parallel"))]
    {
        unsafe { mclBnG1_mulVec(out, points.as_ptr(), scalars.as_ptr(), len) }
    }
}

pub fn is_valid_order(g1: &G1) -> bool {
    unsafe { mclBnG1_isValidOrder(g1) == 1 }
}

#[derive(Default, Debug, Clone, Copy)]
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
    mcl_methods::mclBn_getFpByteSize(),
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

impl G1 {
    pub fn from_arr_64(u: &[[u64; 6]; 3]) -> G1 {
        let mut g1 = G1::default();
        g1.x.set_str(&arr64_6_to_g1_sum(&u[0]).to_string(), 10);
        g1.y.set_str(&arr64_6_to_g1_sum(&u[1]).to_string(), 10);
        g1.z.set_str(&arr64_6_to_g1_sum(&u[2]).to_string(), 10);
        g1
    }

    pub const G1_IDENTITY: G1 = G1 {
        x: Fp {
            d: [0, 0, 0, 0, 0, 0],
        },
        y: Fp {
            d: [0, 0, 0, 0, 0, 0],
        },
        z: Fp {
            d: [0, 0, 0, 0, 0, 0],
        },
    };
}
