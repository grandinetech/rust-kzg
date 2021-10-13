use std::os::raw::c_int;


pub enum CurveType {
    BN254 = 0,
    BN381 = 1,
    SNARK = 4,
    BLS12_381 = 5,
}

const MCLBN_FP_UNIT_SIZE: usize = 6;
const MCLBN_FR_UNIT_SIZE: usize = 4;
const MCLBN_COMPILED_TIME_VAR: c_int = MCLBN_FR_UNIT_SIZE as c_int * 10 + MCLBN_FP_UNIT_SIZE as c_int;


#[macro_escape] pub mod init_def;
pub mod mlc_methods;
pub mod utilities;
pub mod data_types {
    pub mod fr;
    pub mod fp;
    pub mod fp2;
    pub mod g1;
    pub mod g2;
    pub mod gt;
}
pub mod kzg10;
pub mod fk20_fft;
pub mod zero_poly;

pub mod old;
