use crate::data_types::g1::G1;
use crate::BlstP1;
use crate::data_converter::fp_converter::*;

pub fn g1FromBlst(blstG1: BlstP1) -> G1 {
    let mut result = G1::default();
    result.x = fpFromBlst(blstG1.x);
    result.y = fpFromBlst(blstG1.y);
    result.z = fpFromBlst(blstG1.z);
    return result;
}

pub fn g1ToBlst(g1: G1) -> BlstP1 {
    let mut result = BlstP1::default();
    result.x = fpToBlst(g1.x);
    result.y = fpToBlst(g1.y);
    result.z = fpToBlst(g1.z);
    return result;
}