use crate::data_types::g2::G2;
use crate::BlstP2;
use crate::data_converter::fp2_converter::*;

pub fn g2_from_blst(blst_g2: BlstP2) -> G2 {
    G2 { x: fp2_from_blst(blst_g2.x), y: fp2_from_blst(blst_g2.y), z: fp2_from_blst(blst_g2.z) }
}

pub fn g2_to_blst(g2: G2) -> BlstP2 {
    blst::blst_p2 { x: fp2_to_blst(g2.x), y: fp2_to_blst(g2.y), z: fp2_to_blst(g2.z) }
}