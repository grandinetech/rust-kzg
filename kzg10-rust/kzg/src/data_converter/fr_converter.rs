use crate::data_types::fr::Fr;
use crate::BlstFr;

pub fn frFromBlst(fr: BlstFr) -> Fr {
    let mut result = Fr::default();
    for i in 0..4 {
        result.d[i] = fr.l[i];
    }
    return result;
}

pub fn frToBlst(fr: Fr) -> BlstFr {
    let mut result = BlstFr::default();
    for i in 0..4 {
        result.l[i] = fr.d[i];
    }
    return result;
}