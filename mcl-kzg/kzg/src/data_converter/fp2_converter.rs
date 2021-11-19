use crate::data_types::fp2::Fp2;
use crate::BlstFp2;
use crate::data_converter::fp_converter::*;

pub fn fp2_from_blst(fp2: BlstFp2) -> Fp2 {
    let mut result = Fp2::default();
    
    for i in 0..2 {
        result.d[i] = fp_from_blst(fp2.fp[i]);
    }
    
    result
}

pub fn fp2_to_blst(fp2: Fp2) -> BlstFp2 {
    let mut result = BlstFp2::default();
    
    for i in 0..2 {
        result.fp[i] = fp_to_blst(fp2.d[i].clone()); //probably should implement clone
    }
    
    result
}