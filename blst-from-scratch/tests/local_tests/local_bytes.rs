use kzg::Fr;
// use blst_from_scratch::bytes::reverse_bit_order;
use blst_from_scratch::kzg_types::FsFr;

pub fn test_reverse_bit_order_fr() {
    let size = 10;
    let n = 1 << size;

    for i in 0..n {
        let _tmp = FsFr::from_u64(i);

    }

    // reverse_bit_order(a);
}

pub fn test_reverse_bit_order_g1() {

}

pub fn test_reverse_bit_order_fr_large() {

}