use crate::data_types::fp::Fp;
use crate::BlstFp;

pub fn fpFromBlst(fp: BlstFp) -> Fp {
    let mut result = Fp::default();
    for i in 0..4 {
        result.d[i] = fp.l[i];
    }
    return result;
}

pub fn fpToBlst(fp: Fp) -> BlstFp {
    let mut result = BlstFp::default();
    for i in 0..4 {
        result.l[i] = fp.d[i];
    }
    return result;
}