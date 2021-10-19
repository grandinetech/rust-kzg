use crate::data_types::fr::Fr;
use crate::BlstFr;

pub fn frFromBlst(fr: BlstFr) -> Fr {
    let mut intValue: u64 = 0;
    let frBlst = fr;
    unsafe {
        blst::blst_uint64_from_fr(&mut intValue, &frBlst);
    }
    let i32Value = intValue as i32;//could cause problems
    return Fr::from_int(i32Value);
    
    // let mut result = Fr::default();
    // for i in 0..4 {
    //     result.d[i] = fr.l[i];
    // }
    // return result;
}

pub fn frToBlst(fr: Fr) -> BlstFr {
    let mut result = BlstFr::default();
    for i in 0..4 {
        result.l[i] = fr.d[i];
    }
    return result;
}