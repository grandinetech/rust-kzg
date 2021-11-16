use crate::utils::log2_pow2;

pub fn reverse_bit_order<T>(values: &mut Vec<T>) {
    let unused_bit_len = 32 - log2_pow2(values.len());

    for i in 0..values.len() {
        let r = i.reverse_bits() >> unused_bit_len;
        if r > i {
            values.swap(i, r);
        }
    }
}