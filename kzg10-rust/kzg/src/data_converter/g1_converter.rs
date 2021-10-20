use crate::data_types::g1::G1;
use crate::BlstP1;
use crate::data_converter::fp_converter::*;

pub fn g1_from_blst(blstG1: BlstP1) -> G1 {
    let mut result = G1::default();
    result.x = fp_from_blst(blstG1.x);
    result.y = fp_from_blst(blstG1.y);
    result.z = fp_from_blst(blstG1.z);
    return result;
}

pub fn g1_to_blst(g1: G1) -> BlstP1 {
    let mut result = BlstP1::default();
    result.x = fp_to_blst(g1.x);
    result.y = fp_to_blst(g1.y);
    result.z = fp_to_blst(g1.z);
    return result;
}