use crate::data_types::fp::Fp;
use crate::BlstFp;

pub fn fp_from_blst(fp: BlstFp) -> Fp {
    let mut intValue: u64 = 0;
    let fpblst = fp;
    unsafe{
        blst::blst_uint64_from_fp(&mut intValue, &fpblst)
    }
    let i32Value = intValue as i32;
    return Fp::from_int(i32Value);
/*
    let mut result = Fp::default();
    for i in 0..4 {
        result.d[i] = fp.l[i];
    }
    return result;*/
}

pub fn fpToBlst(fp: Fp) -> BlstFp {
    let mut result = BlstFp::default();
    for i in 0..4 {
        result.l[i] = fp.d[i];
    }
    return result;
}