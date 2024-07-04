use core::mem::size_of;

use crate::{G1Affine, G1Fp, G1GetFp, Scalar256, G1};

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct P1XYZZ<TFp: G1Fp> {
    pub x: TFp,
    pub y: TFp,
    pub zzz: TFp,
    pub zz: TFp,
}

#[inline(always)]
pub fn type_zero<T>(ret: &mut T) {
    let rp = ret as *mut T as *mut u64;
    let num = size_of::<T>() / size_of::<u64>();

    for i in 0..num {
        unsafe {
            *rp.wrapping_add(i) = 0;
        }
    }
}

pub const fn is_zero(val: u64) -> u64 {
    (!val & (val.wrapping_sub(1))) >> (u64::BITS - 1)
}

#[inline(always)]
pub fn vec_zero_rt(ret: *mut u64, mut num: usize) {
    num /= size_of::<usize>();
    for i in 0..num {
        unsafe {
            *ret.add(i) = 0;
        }
    }
}

#[inline(always)]
pub fn vec_is_zero(a: *const u8, num: usize) -> u64 {
    let ap = a as *const u64;
    let num = num / size_of::<u64>();

    let mut acc: u64 = 0;
    for i in 0..num {
        unsafe {
            acc |= *ap.wrapping_add(i);
        }
    }

    is_zero(acc)
}

#[inline(always)]
pub fn type_is_zero<T>(a: &T) -> u64 {
    let ap = a as *const T as *const u64;
    let num = size_of::<T>() / size_of::<u64>();

    let mut acc: u64 = 0;
    for i in 0..num {
        unsafe {
            acc |= *ap.wrapping_add(i);
        }
    }

    is_zero(acc)
}

#[inline(always)]
fn vec_copy(ret: *mut u8, a: *const u8, num: usize) {
    let rp = ret as *mut u64;
    let ap = a as *const u64;

    let num = num / size_of::<u64>();

    for i in 0..num {
        unsafe {
            *rp.wrapping_add(i) = *ap.wrapping_add(i);
        }
    }
}

pub fn p1_to_jacobian<TG1: G1 + G1GetFp<TFp>, TFp: G1Fp>(out: &mut TG1, input: &P1XYZZ<TFp>) {
    *out.x_mut() = input.x.mul_fp(&input.zz);
    *out.y_mut() = input.y.mul_fp(&input.zzz);
    *out.z_mut() = input.zz;
}

fn p1_dadd_affine<TG1: G1, TFp: G1Fp, TG1Affine: G1Affine<TG1, TFp>>(
    out: &mut P1XYZZ<TFp>,
    p2: &TG1Affine,
    subtract: bool, // Need to replace this somehow
) {
    if type_is_zero(p2) != 0 {
        return;
    } else if vec_is_zero(&out.zzz as *const TFp as *const u8, 2 * size_of::<TFp>()) != 0 {
        vec_copy(
            &mut (out.x) as *mut TFp as *mut u8,
            ((*p2).x()) as *const TFp as *const u8,
            2 * size_of::<TFp>(),
        );

        out.zzz = TFp::bls12_381_rx_p();
        if subtract {
            out.zzz.neg_assign();
        }

        out.zz = TFp::bls12_381_rx_p();
        return;
    }

    let mut p = p2.x().mul_fp(&out.zz);
    let mut r = p2.y().mul_fp(&out.zzz);
    if subtract {
        r.neg_assign();
    }
    p.sub_assign_fp(&out.x);
    r.sub_assign_fp(&out.y);
    if type_is_zero(&p) == 0 {
        let pp = p.square();
        let ppp = pp.mul_fp(&p);
        let mut q = out.x.mul_fp(&pp);
        out.x = r.square();
        p = q.add_fp(&q);
        out.x.sub_assign_fp(&ppp);
        out.x.sub_assign_fp(&p);
        q.sub_assign_fp(&out.x);
        q.mul_assign_fp(&r);
        out.y.mul_assign_fp(&ppp);
        out.y = q.sub_fp(&out.y);
        out.zz.mul_assign_fp(&pp);
        out.zzz.mul_assign_fp(&ppp);
    } else if type_is_zero(&r) != 0 {
        let mut u = p2.y().add_fp(p2.y());
        out.zz = u.square();
        out.zzz = out.zz.mul_fp(&u);
        let mut s = p2.x().mul_fp(&out.zz);
        let mut m = p2.x().square();
        m = m.add_fp(&m).add_fp(&m);
        out.x = m.square();
        u = s.add_fp(&s);
        out.x.sub_assign_fp(&u);
        out.y = out.zzz.mul_fp(p2.y());
        s.sub_assign_fp(&out.x);
        s.mul_assign_fp(&m);
        out.y = s.sub_fp(&out.y);
        if subtract {
            out.zzz.neg_assign();
        }
    } else {
        vec_zero_rt(
            &mut out.zzz as *mut TFp as *mut u64,
            2 * core::mem::size_of_val(&out.zzz),
        );
    }
}

pub fn p1_dadd<TFp: G1Fp>(out: &mut P1XYZZ<TFp>, p2: &P1XYZZ<TFp>) {
    if vec_is_zero(&p2.zzz as *const TFp as *const u8, 2 * size_of::<TFp>()) != 0 {
        return;
    } else if vec_is_zero(&out.zzz as *const TFp as *const u8, 2 * size_of::<TFp>()) != 0 {
        *out = *p2;
        return;
    }

    let mut u = out.x.mul_fp(&p2.zz);
    let mut s = out.y.mul_fp(&p2.zzz);
    let mut p = p2.x.mul_fp(&out.zz);
    let mut r = p2.y.mul_fp(&out.zzz);

    p.sub_assign_fp(&u);
    r.sub_assign_fp(&s);

    if type_is_zero(&p) == 0 {
        let pp = p.square();
        let ppp = pp.mul_fp(&p);
        let mut q = u.mul_fp(&pp);
        out.x = r.square();
        p = q.add_fp(&q);
        out.x.sub_assign_fp(&ppp);
        out.x.sub_assign_fp(&p);
        q.sub_assign_fp(&out.x);
        q.mul_assign_fp(&r);
        out.y = s.mul_fp(&ppp);
        out.y = q.sub_fp(&out.y);
        out.zz.mul_assign_fp(&p2.zz);
        out.zzz.mul_assign_fp(&p2.zzz);
        out.zz.mul_assign_fp(&pp);
        out.zzz.mul_assign_fp(&ppp);
    } else if type_is_zero(&r) != 0 {
        u = out.y.add_fp(&out.y);
        let v = u.square();
        let w = v.mul_fp(&u);
        s = out.x.mul_fp(&v);
        let mut m = out.x.square();
        m = m.add_fp(&m).add_fp(&m);
        out.x = m.square();
        u = s.add_fp(&s);
        out.x.sub_assign_fp(&u);
        out.y = w.mul_fp(&out.y);
        s.sub_assign_fp(&out.x);
        s.mul_assign_fp(&m);
        out.y = s.sub_fp(&out.y);
        out.zz.mul_assign_fp(&v);
        out.zzz.mul_assign_fp(&w);
    } else {
        vec_zero_rt(&mut out.zzz as *mut TFp as *mut u64, 2 * size_of::<TFp>());
    }
}

/// Extract `bits` from the beginning of `d` array, with offset `off`.
///
/// This function is used to extract N bits from the scalar, decomposing it into q-ary representation.
/// This works because `q` is `2^bits`, so extracting `bits` from scalar will break it into the correct representation.
///
/// Caution! This function guarantees only that `bits` amount of bits from the right will be extracted. All unused bits
/// will contain "trash". For example, if we try to extract first 4 bits from the array `[0b01010111u8]`, this
/// function will return `0111`, but other bits will contain trash (see tests::get_wval_limb_example_1)
///
/// # Arguments
///
/// * `d`    - byte array, from which bits will be extracted
/// * `off`  - index of first bit, that will be extracted
/// * `bits` - number of bits to extract (up to 25)
///
/// # Examples
///
/// See tests::get_wval_limb_example_2
///
pub fn get_wval_limb(d: &Scalar256, off: usize, bits: usize) -> u64 {
    let mut d = d.as_u8();
    let top = ((off + bits - 1) / 8).wrapping_sub((off / 8).wrapping_sub(1));
    d = &d[off / 8..];
    let mut mask = u64::MAX;
    let mut ret: u64 = 0;
    for i in 0..4usize {
        ret |= (d[0] as u64 & mask) << (8 * i);

        mask = 0u64.wrapping_sub(((i + 1).wrapping_sub(top) >> (usize::BITS - 1)) as u64);
        d = &d[(1 & mask).try_into().unwrap()..];
    }
    ret >> (off % 8)
}

/// Window value encoding that utilizes the fact that -P is trivially
/// calculated, which allows to halve the size of the pre-computed table,
/// is attributed to A. D. Booth, hence the name of the subroutines...
///
/// TODO: figure out how this function works exactly
pub const fn booth_encode(wval: u64, sz: usize) -> u64 {
    let mask = 0u64.wrapping_sub(wval >> sz);

    let wval = (wval + 1) >> 1;
    (wval ^ mask).wrapping_sub(mask)
}

/// Decode bucket index and move point to corresponding bucket
///
/// This method will decode `booth_idx`, and add or subtract point to bucket.
/// booth_idx contains bucket index and sign. Sign shows, if point needs to be added to or subtracted from bucket.
///
/// ## Arguments:
///
/// * buckets   - pointer to the bucket array beginning
/// * booth_idx - bucket index, encoded with [booth_encode] function
/// * wbits     - window size, aka exponent of q (q^window)
/// * point     - point to move
///
pub fn booth_decode<TG1: G1, TFp: G1Fp, TG1Affine: G1Affine<TG1, TFp>>(
    buckets: &mut [P1XYZZ<TFp>],
    mut booth_idx: u64,
    wbits: usize,
    p: &TG1Affine,
) {
    let booth_sign: bool = ((booth_idx >> wbits) & 1) != 0;
    booth_idx &= (1 << wbits) - 1;
    if booth_idx != 0 {
        p1_dadd_affine(&mut buckets[(booth_idx - 1) as usize], p, booth_sign);
    }
}

pub const fn num_bits(l: usize) -> usize {
    8 * core::mem::size_of::<usize>() - l.leading_zeros() as usize
}

/// Function, which approximates minimum of this function:
/// y = ceil(255/w) * (npoints + 2^w + w + 1)
/// This function is number of additions and doublings required to compute msm using Pippenger algorithm.
/// Parts of this function:
///   ceil(255/w) - how many parts will be in decomposed scalar. Scalar width is 255 bits, so converting it into q-ary
///                 representation, will produce 255/w parts. q-ary representation, where q = 2^w, for scalar a is:
///                 a = a_1 + a_2 * q + ... + a_n * q^(ceil(255/w)).
///   npoints     - each scalar must be assigned to a bucket (bucket accumulation). Assigning point to bucket means
///                 adding it to existing point in bucket - hence, the addition.
///   2^w         - computing total bucket sum (bucket aggregation). Total number of buckets (scratch size) is 2^(w-1).
///                 Adding each point to total bucket sum requires 2 point addition operations, so 2 * 2^(w-1) = 2^w.
///   w + 1       - each bucket sum must be multiplied by 2^w. To do this, we need w doublings. Adding this sum to the
///                 total requires one more point addition, hence +1.
pub const fn pippenger_window_size(npoints: usize) -> usize {
    let wbits = num_bits(npoints);

    if wbits > 13 {
        return wbits - 4;
    }
    if wbits > 5 {
        return wbits - 3;
    }
    2
}

#[cfg(test)]
mod tests {
    use crate::{
        msm::pippenger_utils::{booth_encode, get_wval_limb},
        Scalar256,
    };

    #[test]
    fn booth_encode_must_produce_correct_results() {
        assert_eq!(booth_encode(0, 1), 0);
        assert_eq!(booth_encode(0, 5), 0);
        assert_eq!(booth_encode(1, 1), 1);
        assert_eq!(booth_encode(55, 5), 18446744073709551588);
    }

    #[test]
    fn get_wval_limb_example_1() {
        let val = get_wval_limb(
            &Scalar256 {
                data: [0b01010111u64, 0u64, 0u64, 0u64],
            },
            0,
            4,
        );
        assert_eq!(val, 0b01010111);
        // if you want to get value containing only extracted bits and zeros, do bitwise and on return value with mask:
        assert_eq!(val & 0b00001111, 0b00000111);
    }

    #[test]
    fn get_wval_limb_example_2() {
        // Scalars are represented with 32 bytes. To simplify example, let's say our scalars are only 4 bytes long.
        // Then, we can take `q` as `2^6`. Then consider scalar value `4244836224`, bytes: `[128u8, 15u8, 3u8, 253u8]`
        // (little-endian order). So if we repeatedly take 6 bits from this scalar, we will get q-ary representation
        // of this scalar:

        // this is [128u8, 15u8, 3u8, 253u8] written in binary
        let scalar = Scalar256 {
            data: [0b11111101000000110000111110000000u64, 0u64, 0, 0],
        };
        let limb_1 = get_wval_limb(&scalar, 0, 6);
        // function leaves trash on all other bytes, so real answer only in 6 bits from right
        assert_eq!(limb_1 & 0b00111111, 0b00000000 /*  0 */); // 11111101000000110000111110|000000|
        let limb_2 = get_wval_limb(&scalar, 6, 6);
        assert_eq!(limb_2 & 0b00111111, 0b00111110 /* 62 */); // 11111101000000110000|111110|000000
        let limb_3 = get_wval_limb(&scalar, 12, 6);
        assert_eq!(limb_3 & 0b00111111, 0b00110000 /* 48 */); // 11111101000000|110000|111110000000
        let limb_4 = get_wval_limb(&scalar, 18, 6);
        assert_eq!(limb_4 & 0b00111111, 0b00000000 /*  0 */); // 11111101|000000|110000111110000000
        let limb_5 = get_wval_limb(&scalar, 24, 6);
        assert_eq!(limb_5 & 0b00111111, 0b00111101 /* 61 */); // 11|111101|000000110000111110000000
        let limb_r = get_wval_limb(&scalar, 28, 8 % 6); // get remaining part
        assert_eq!(limb_r & 0b00000011, 0b00000011 /*  3 */); // |11|111101000000110000111110000000

        // This gives q-ary representation of scalar `4244836224`, where `q` = `2^6` = `64`:
        // 4244836224 = 0 * 64^0 + 62 * 64^1 + 48 * 64^2 + 0 * 64^3 + 61 * 64^4 + 3 * 64^5
    }
}
