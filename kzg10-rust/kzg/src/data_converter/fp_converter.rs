use crate::data_types::fp::Fp;
use crate::BlstFp;

pub fn fp_from_blst(fp: BlstFp) -> Fp {
    let mut int_value: u64 = 0;
    let fp_blst = fp;
    unsafe{
        blst::blst_uint64_from_fp(&mut int_value, &fp_blst)
    }
    let i32_value = int_value as i32;
    return Fp::from_int(i32_value);

    /*
    let mut result = Fp::default();
    for i in 0..4 {
        result.d[i] = fp.l[i];
    }
    return result;*/
}

pub fn fp_to_blst(fp: Fp) -> BlstFp {
    let mut result = BlstFp::default();
    for i in 0..4 {
        result.l[i] = fp.d[i];
    }
    return result;
}