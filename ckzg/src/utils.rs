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
    return n & (n - 1) == 0;
}

pub fn next_pow_of_2(x: usize) -> usize {
    if x == 0 {
        return 1;
    }
    if is_power_of_2(x) {
        return x;
    }
    return 1 << (log_2(x) + 1);
}

pub fn log_2_byte(b: u8) -> usize {
    let mut r = if b > 0xF { 1 } else { 0 } << 2;
    let mut b = b >> r;
    let shift = if b > 0x3 { 1 } else { 0 } << 1;
    b >>= shift + 1;
    r |= shift | b;
    r.into()
}

pub fn reverse_bits_limited(length: usize, value: usize) -> usize {
    let unused_bits = length.leading_zeros();
    value.reverse_bits() >> unused_bits
}
