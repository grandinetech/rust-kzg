pub fn reverse_bit_order<T>(values: &mut Vec<T>) where T: Clone {
    let unused_bit_len = values.len().leading_zeros() + 1;
    for i in 0..values.len() - 1 {
        let r = i.reverse_bits() >> unused_bit_len;
        if r > i {
            let tmp = values[r].clone();
            values[r] = values[i].clone();
            values[i] = tmp;
        }
    }
}