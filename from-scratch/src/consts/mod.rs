use kzg::G1;
use blst::blst_fp;

pub static G1_IDENTITY: G1 = G1 {
    x: blst_fp{l: [0,0,0,0,0,0]},
    y: blst_fp{l: [0,0,0,0,0,0]},
    z: blst_fp{l: [0,0,0,0,0,0]},
};
