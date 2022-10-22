use std::os::raw::c_int;

pub type BlstPairing = blst::Pairing;
pub type BlstFp = blst::blst_fp;
pub type BlstFp2 = blst::blst_fp2;
pub type BlstFp12 = blst::blst_fp12;
pub type BlstFp6 = blst::blst_fp6;
pub type BlstFr = blst::blst_fr;
pub type BlstP1 = blst::blst_p1;
pub type BlstP1Affine = blst::blst_p1_affine;
pub type BlstP2 = blst::blst_p2;
pub type BlstP2Affine = blst::blst_p2_affine;
pub type BlstScalar = blst::blst_scalar;
pub type BlstUniq = blst::blst_uniq;

pub enum CurveType {
    BN254 = 0,
    BN381 = 1,
    SNARK = 4,
    BLS12_381 = 5,
}

const MCLBN_FP_UNIT_SIZE: usize = 6;
const MCLBN_FR_UNIT_SIZE: usize = 4;
const MCLBN_COMPILED_TIME_VAR: c_int =
    MCLBN_FR_UNIT_SIZE as c_int * 10 + MCLBN_FP_UNIT_SIZE as c_int;

#[macro_use]
pub mod init_def;
pub mod mcl_methods;
pub mod utilities;
pub mod data_types {
    pub mod fp;
    pub mod fp2;
    pub mod fr;
    pub mod g1;
    pub mod g2;
    pub mod gt;
}
pub mod trait_implementations {
    pub mod das;
    pub mod fft_fr;
    pub mod fft_g1;
    pub mod fft_settings;
    pub mod fr;
    pub mod poly;
    pub mod zero_poly;
    pub mod g1;
    pub mod g2;
    pub mod kzg_settings;
    pub mod fk20;
}
pub mod das;
pub mod data_recovery;
pub mod eip_4844;
pub mod fk20_fft;
pub mod fk20_matrix;
pub mod kzg10;
pub mod zero_poly;
pub mod kzg_settings;
