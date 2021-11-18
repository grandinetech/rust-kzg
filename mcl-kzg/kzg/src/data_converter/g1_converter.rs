use crate::data_types::g1::G1;
use crate::BlstP1;
use crate::data_converter::fp_converter::*;

pub fn g1_from_blst(blst_g1: BlstP1) -> G1 {
    G1 { x: fp_from_blst(blst_g1.x), y: fp_from_blst(blst_g1.y), z: fp_from_blst(blst_g1.z) }
}

pub fn g1_to_blst(g1: G1) -> BlstP1 {
    blst::blst_p1 { x: fp_to_blst(g1.x), y: fp_to_blst(g1.y), z: fp_to_blst(g1.z) }
}