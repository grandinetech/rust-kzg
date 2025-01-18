extern crate alloc;

use alloc::string::String;
use core::mem;

pub fn reverse_bit_order<T>(vals: &mut [T]) -> Result<(), String>
where
    T: Clone,
{
    if vals.is_empty() {
        return Err(String::from("Values can not be empty"));
    }

    // required for tests
    if vals.len() == 1 {
        return Ok(());
    }

    if !vals.len().is_power_of_two() {
        return Err(String::from("Values length has to be a power of 2"));
    }

    let unused_bit_len = vals.len().leading_zeros() + 1;
    for i in 0..vals.len() - 1 {
        let r = i.reverse_bits() >> unused_bit_len;
        if r > i {
            let tmp = vals[r].clone();
            vals[r] = vals[i].clone();
            vals[i] = tmp;
        }
    }

    Ok(())
}

pub fn log_2_byte(b: u8) -> usize {
    let mut r = u8::from(b > 0xF) << 2;
    let mut b = b >> r;
    let shift = u8::from(b > 0x3) << 1;
    b >>= shift + 1;
    r |= shift | b;
    r.into()
}

pub fn log2_pow2(n: usize) -> usize {
    let bytes: [usize; 5] = [0xAAAAAAAA, 0xCCCCCCCC, 0xF0F0F0F0, 0xFF00FF00, 0xFFFF0000];
    let mut r: usize = usize::from((n & bytes[0]) != 0);
    r |= usize::from((n & bytes[1]) != 0) << 1;
    r |= usize::from((n & bytes[2]) != 0) << 2;
    r |= usize::from((n & bytes[3]) != 0) << 3;
    r |= usize::from((n & bytes[4]) != 0) << 4;
    r
}

pub fn log2_u64(n: usize) -> usize {
    let mut n2 = n;
    let mut r: usize = 0;
    while (n2 >> 1) != 0 {
        n2 >>= 1;
        r += 1;
    }
    r
}

const fn num_bits<T>() -> usize {
    mem::size_of::<T>() * 8
}

pub fn log_2(x: usize) -> usize {
    if x == 0 {
        return 0;
    }
    num_bits::<usize>() - (x.leading_zeros() as usize) - 1
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

pub fn is_power_of_two(n: usize) -> bool {
    n & (n - 1) == 0
}

pub fn reverse_bits_limited(length: usize, value: usize) -> usize {
    let unused_bits = length.leading_zeros() + 1;
    value.reverse_bits() >> unused_bits
}

#[macro_export]
macro_rules! cfg_iter_mut {
    ($collection:expr) => {{
        #[cfg(feature = "parallel")]
        {
            $collection.par_iter_mut()
        }
        #[cfg(not(feature = "parallel"))]
        {
            $collection.iter_mut()
        }
    }};
}

#[macro_export]
macro_rules! cfg_iter {
    ($collection:expr) => {{
        #[cfg(feature = "parallel")]
        {
            $collection.par_iter()
        }
        #[cfg(not(feature = "parallel"))]
        {
            $collection.iter()
        }
    }};
}

#[macro_export]
macro_rules! cfg_chunks {
    ($collection:expr, $chunk_size:expr) => {{
        #[cfg(feature = "parallel")]
        {
            $collection.par_chunks($chunk_size)
        }
        #[cfg(not(feature = "parallel"))]
        {
            $collection.chunks($chunk_size)
        }
    }};
}
