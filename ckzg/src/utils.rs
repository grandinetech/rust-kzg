const fn num_bits<T>() -> usize {
    std::mem::size_of::<T>() * 8
}

pub fn log_2(x: usize) -> usize {
    if x == 0 {
        return 0;
    }
    num_bits::<usize>() as usize - (x.leading_zeros() as usize) - 1
}

pub fn is_power_of_2(n: usize) -> bool {
    n & (n - 1) == 0
}

pub fn next_pow_of_2(x: usize) -> usize {
    if x == 0 {
        return 1;
    }
    if is_power_of_2(x) {
        return x;
    }
    1 << (log_2(x) + 1)
}

pub fn log_2_byte(b: u8) -> usize {
    let mut r = if b > 0xF { 1 } else { 0 } << 2;
    let mut b = b >> r;
    let shift = if b > 0x3 { 1 } else { 0 } << 1;
    b >>= shift + 1;
    r |= shift | b;
    r.into()
}

pub fn reverse_bit_order<T>(values: &mut [T])
where
    T: Clone,
{
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
