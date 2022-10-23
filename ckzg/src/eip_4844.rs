use std::convert::TryInto;

use crate::finite::BlstFr;

use kzg::Fr;

pub fn bytes_from_bls_field(out: &mut [u8; 32usize], fr: &BlstFr) {
    let v = &fr.to_u64_arr();
    // investigate if being little endian changes something
    // order of bytes might need to be reversed
    let my_u8_vec_bis:Vec<u8> = unsafe{
        (v[..4].align_to::<u8>().1).to_vec()
    };
    *out = my_u8_vec_bis.try_into().unwrap();

}

pub fn bytes_to_bls_field(out: &mut BlstFr, bytes: [u8; 32usize]) {
    let mut my_u64_vec:Vec<u64> = Vec::default();
    unsafe{
        my_u64_vec = (bytes[..32].align_to::<u64>().1).to_vec();
    }
    let arr: [u64; 4] = match my_u64_vec.try_into(){
        Ok(arr) => arr,
        Err(_) => panic!()
    };
    *out = BlstFr::from_u64_arr(&arr);
}

pub fn compute_powers(base: &BlstFr, num_powers: usize) -> Vec<BlstFr> {
    let mut powers: Vec<BlstFr> = vec![BlstFr::default(); num_powers];
    powers[0] = BlstFr::one();
    for i in 1..num_powers {
        powers[i] = powers[i - 1].mul(base);
    }
    powers
}