use crate::data_types::gt::GT;
use crate::BlstFp12;
use crate::data_converter::fp_converter::*;

pub fn gtFromBlst(fp12: BlstFp12) -> GT {
    let mut result = GT::default();
    
    for i in 0..2 {
        for j in 0..3 {
            for k in 0..2 {
                result.d[i*6+j*3+k] = fpFromBlst(fp12.fp6[i].fp2[j].fp[k]);
            }
        }
    }
    
    return result;
}

pub fn gtToBlst(fp12: GT) -> BlstFp12 {
    let mut result = BlstFp12::default();
    
    for i in 0..2 {
        for j in 0..3 {
            for k in 0..2 {
                result.fp6[i].fp2[j].fp[k] = fpToBlst(fp12.d[i*6+j*3+k].clone()); //probably should implement clone
            }
        }
    }
    
    return result;
}