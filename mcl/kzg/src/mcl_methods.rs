use crate::data_types::fr::Fr;
use crate::data_types::g1::G1;
use crate::data_types::g2::G2;
use crate::data_types::gt::GT;
use crate::CurveType;
use std::os::raw::c_int;

#[link(name = "mcl", kind = "static")]
#[link(name = "mclbn384_256", kind = "static")]
#[link(name = "stdc++")]
#[allow(non_snake_case)]
extern "C" {
    // global functions
    pub fn mclBn_init(curve: c_int, compiledTimeVar: c_int) -> c_int;
    pub fn mclBn_getVersion() -> u32;
    pub fn mclBn_getFrByteSize() -> u32;
    pub fn mclBn_getFpByteSize() -> u32;
    pub fn mclBn_getCurveOrder(buf: *mut u8, maxBufSize: usize) -> usize;
    pub fn mclBn_getFieldOrder(buf: *mut u8, maxBufSize: usize) -> usize;
    pub fn mclBn_pairing(z: *mut GT, x: *const G1, y: *const G2);
    pub fn mclBn_millerLoop(z: *mut GT, x: *const G1, y: *const G2);
    pub fn mclBn_finalExp(y: *mut GT, x: *const GT);
    pub fn mclBn_FrEvaluatePolynomial(result: *mut Fr, poly: *const Fr, bufSize: usize, x: *const Fr);
    pub fn mclBn_setETHserialization(v: i32);
}

pub fn get_version() -> u32 {
    unsafe { mclBn_getVersion() }
}

pub fn set_eth_serialization(v: i32) {
    unsafe { mclBn_setETHserialization(v) }
}

pub fn init(curve: CurveType) -> bool {
    unsafe { mclBn_init(curve as c_int, crate::MCLBN_COMPILED_TIME_VAR) == 0 }
}

pub fn get_fr_serialized_size() -> u32 {
    unsafe { mclBn_getFrByteSize() }
}

pub fn get_fp_serialized_size() -> u32 {
    unsafe { mclBn_getFpByteSize() }
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
