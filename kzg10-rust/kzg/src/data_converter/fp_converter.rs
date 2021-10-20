use crate::data_types::fp::Fp;
use crate::BlstFp;

pub fn fp_from_blst(fp: BlstFp) -> Fp {
    let mut result = Fp::default();
    for i in 0..4 {
        result.d[i] = fp.l[i];
    }
    return result;
}

pub fn fp_to_blst(fp: Fp) -> BlstFp {
    let mut result = BlstFp::default();
    for i in 0..4 {
        result.l[i] = fp.d[i];
    }
    return result;
}