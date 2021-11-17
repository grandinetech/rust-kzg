use crate::data_types::fr::Fr;
use crate::BlstFr;

pub fn fr_from_blst(fr: BlstFr) -> Fr {
    // let mut int_value: u64 = 0;
    // let fr_blst = fr;
    // unsafe {
    //     blst::blst_uint64_from_fr(&mut int_value, &fr_blst);
    // }
    // let i32_value = int_value as i32;//could cause problems
    // return Fr::from_int(i32_value);
    
    let mut result = Fr::default();
    for i in 0..4 {
        result.d[i] = fr.l[i];
    }
    return result;
}

pub fn fr_to_blst(fr: Fr) -> BlstFr {
    let mut result = BlstFr::default();
    for i in 0..4 {
        result.l[i] = fr.d[i];
    }
    return result;
}