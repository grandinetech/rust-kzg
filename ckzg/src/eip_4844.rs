use std::convert::TryInto;

use crate::finite::BlstFr;

use kzg::Fr;

pub fn bytes_from_bls_field(fr: &BlstFr) -> [u8; 32usize]{
    let v = &fr.to_u64_arr();
    // investigate if being little endian changes something
    // order of bytes might need to be reversed
    let my_u8_vec_bis:Vec<u8> = unsafe{
        (v[..4].align_to::<u8>().1).to_vec()
    };
    my_u8_vec_bis.try_into().unwrap()
}

pub fn bytes_to_bls_field(bytes: &[u8; 32usize]) -> BlstFr{
    let my_u64_vec = unsafe{
        (bytes[..32].align_to::<u64>().1).to_vec()
    };
    let arr: [u64; 4] = match my_u64_vec.try_into(){
        Ok(arr) => arr,
        Err(_) => panic!()
    };
    BlstFr::from_u64_arr(&arr)
}

pub fn compute_powers(base: &BlstFr, num_powers: usize) -> Vec<BlstFr> {
    let mut powers: Vec<BlstFr> = vec![BlstFr::default(); num_powers];
    powers[0] = BlstFr::one();
    for i in 1..num_powers {
        powers[i] = powers[i - 1].mul(base);
    }
    powers
}