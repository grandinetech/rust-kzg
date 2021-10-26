pub fn is_power_of_two(x: usize) -> bool {
    (x != 0) && ((x & (x - 1)) == 0)
}

pub fn next_power_of_two(v: usize) -> usize {
    let mut v = v;

    v -= 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v |= v >> 32;
    v += 1;
    v += (v == 0) as usize;

    v
}

pub fn log_2_byte(b: u8) -> usize {
    let mut r = if b > 0xF { 1 } else { 0 } << 2;
    let mut b = b >> r;
    let shift = if b > 0x3 { 1 } else { 0 } << 1;
    b >>= shift + 1;
    r |= shift | b;
    r.into()
}

pub fn log2_pow2(n: usize) -> usize {
    let bytes: [usize; 5] = [0xAAAAAAAA, 0xCCCCCCCC, 0xF0F0F0F0, 0xFF00FF00, 0xFFFF0000];
    let mut r: usize = if (n & bytes[0]) != 0 { 1 } else { 0 };
    r |= if (n & bytes[1]) != 0 { 1 } else { 0 } << 1;
    r |= if (n & bytes[2]) != 0 { 1 } else { 0 } << 2;
    r |= if (n & bytes[3]) != 0 { 1 } else { 0 } << 3;
    r |= if (n & bytes[4]) != 0 { 1 } else { 0 } << 4;
    r
}

pub fn log2_u64(n: usize) -> usize {
    let mut n2 = n;
    let mut r: usize = 0;
    while (n2 >> 1) != 0 {
        n2 = n2 >> 1;
        r += 1;
    }
    r
}

pub fn min_u64(a: usize, b: usize) -> Result<usize, String> {
    return if a < b { Ok(a) } else { Ok(b) };
}
