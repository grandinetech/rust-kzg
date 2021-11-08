const fn num_bits<T>() -> usize {
    std::mem::size_of::<T>() * 8
}

pub fn log_2(x: usize) -> usize {
    if x == 0 {
        return 0;
    }
    num_bits::<usize>() as usize - (x.leading_zeros() as usize) - 1
}

fn is_power_of_2(n: usize) -> bool {
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
