use crate::data_types::g1::G1;
use crate::BlstP1;

pub fn g1FromBlst(blstG1: BlstP1) -> G1 {
    let mut result = G1::default();
    result.x = blstG1.x;
    return result;
}

// pub fn g1ToBlst(g1: G1) -> BlstP1 {
//     let mut result = BlstFr::default();
//     for i in 0..4 {
//         result.l[i] = fr.d[i];
//     }
//     return result;
// }