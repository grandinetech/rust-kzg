use crate::data_types::gt::GT;
use crate::BlstFp12;
use crate::data_converter::fp_converter::*;

pub fn fp2FromBlst(fp12: BlstFp12) -> GT {
    let mut result = GT::default();
    
    for i in 0..12 {
        result.d[i] = fpFromBlst(fp12.fp[i]);
    }
    
    return result;
}

pub fn fp2ToBlst(fp12: GT) -> BlstFp12 {
    let mut result = BlstFp12::default();
    
    for i in 0..12 {
        result.fp[i] = fpToBlst(fp12.d[i].clone()); //probably should implement clone
    }
    
    return result;
}