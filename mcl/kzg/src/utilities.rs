use primitive_types::U512;

pub fn order_by_rev_bit_order<T>(vals: &mut [T])
where
    T: Clone,
{
    let unused_bit_len = vals.len().leading_zeros() + 1;
    for i in 0..vals.len() {
        let r = i.reverse_bits() >> unused_bit_len;
        if r > i {
            let tmp = vals[r].clone();
            vals[r] = vals[i].clone();
            vals[i] = tmp;
        }
    }
}

pub fn is_power_of_2(n: usize) -> bool {
    n & (n - 1) == 0
}

const fn num_bits<T>() -> usize {
    std::mem::size_of::<T>() * 8
}

pub fn log_2_byte(x: u8) -> usize {
    log_2(x.into())
}

pub fn log_2(x: usize) -> usize {
    if x == 0 {
        return 0;
    }
    num_bits::<usize>() - (x.leading_zeros() as usize) - 1
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

pub fn reverse_bits_limited(length: usize, value: usize) -> usize {
    let unused_bits = length.leading_zeros();
    value.reverse_bits() >> unused_bits
}

pub fn arr64_6_to_g1_sum(u: &[u64; 6]) -> U512 {
    U512([u[0], u[1], u[2], u[3], u[4], u[5], 0, 0])
}
