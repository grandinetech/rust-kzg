use crate::utils::log2_pow2;

pub(crate) fn reverse_bit_order<T>(vals: &mut Vec<T>) where T: Clone {
    let unused_bit_len = vals.len().leading_zeros() + 1;
    for i in 0..vals.len() - 1 {
        let r = i.reverse_bits() >> unused_bit_len;
        if r > i {
            let tmp = vals[r].clone();
            vals[r] = vals[i].clone();
            vals[i] = tmp;
        }
    }
}