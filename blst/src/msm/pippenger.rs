// The code in this file was ported from blst library https://github.com/supranational/blst
// Original code license could be found here https://github.com/supranational/blst/blob/4a21b61dd40dbe1f411037f1e5eb1c527022eae4/LICENSE

use core::{mem::size_of, ptr};

extern crate alloc;

use alloc::{vec, vec::Vec};

use blst::{
    blst_fp, blst_p1, blst_p1_add, blst_p1_affine, blst_p1_double, blst_p1s_mult_wbits,
    blst_p1s_mult_wbits_precompute, blst_p1s_mult_wbits_scratch_sizeof, blst_scalar,
    blst_scalar_from_fr, byte, limb_t,
};

use crate::types::{fr::FsFr, g1::FsG1};

fn pippenger_window_size(mut npoints: usize) -> usize {
    let mut wbits = 0usize;

    loop {
        npoints >>= 1;
        if npoints == 0 {
            break;
        }
        wbits += 1;
    }

    if wbits > 12 {
        wbits - 3
    } else if wbits > 4 {
        wbits - 2
    } else if wbits != 0 {
        2
    } else {
        1
    }
}

fn is_zero(val: limb_t) -> limb_t {
    (!val & (val.wrapping_sub(1))) >> (limb_t::BITS - 1)
}

/// Window value encoding that utilizes the fact that -P is trivially
/// calculated, which allows to halve the size of pre-computed table,
/// is attributed to A. D. Booth, hence the name of the subroutines...
///
/// TODO: figure out how this function works exactly
fn booth_encode(wval: limb_t, sz: usize) -> limb_t {
    let mask = (0 as limb_t).wrapping_sub(wval >> sz);

    let wval = (wval + 1) >> 1;
    (wval ^ mask).wrapping_sub(mask)
}

#[inline(always)]
unsafe fn vec_zero(ret: *mut limb_t, mut num: usize) {
    num /= size_of::<usize>();

    for i in 0..num {
        *ret.add(i) = 0;
    }
}

/// Extract `bits` from the beginning of `d` array, with offset `off`.
///
/// This function is used to extract N bits from the scalar, decomposing it into q-ary representation.
/// This works because `q` is `2^bits`, so extracting `bits` from scalar will break it to correct representation.
///
/// Caution! This function guarantees only that `bits` bits from the right will contain extracted. All unused bits
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
pub fn get_wval_limb(mut d: &[u8], off: usize, bits: usize) -> limb_t {
    // Calculate topmost byte that needs to be considered.
    let top = ((off + bits - 1) / 8).wrapping_sub((off / 8).wrapping_sub(1));

    // Skipping first `off/8` of bytes, because offset specified how many bits must be ignored
    d = &d[off / 8..];

    // For first iteration, none bits will be ignored - all bits added to result
    let mut mask = limb_t::MAX;

    let mut ret: limb_t = 0;
    for i in 0..4usize {
        /*
         * Add bits from current byte to the result.
         *
         * Applying bitwise and (&) on current byte and mask will keep or ignore all bits from current byte, because
         * mask can only be 0 or limb_t::MAX. Doing right bit shift will move those bits to correct position, e.g. when
         * `i=0` (we are processing first byte), bits won't move, when `i=1` bits will be moved by 8 (1 byte) and so on.
         * After that, we will get value, that is zero-padded from the right and left, so doing bitwise or (|) operation
         * with the result, will just append bytes to it.
         */
        ret |= (d[0] as limb_t & mask) << (8 * i);

        /*
         * Create new mask - either 0 or limb_t::MAX.
         *
         * If `i+1` is greater than or equal to `top`, then byte must be ignored, so the mask is set to `0`. Otherwise,
         * mask is set to `limb_t::MAX` (include all bits). This is done for branch optimization (avoid if).
         */
        mask =
            (0 as limb_t).wrapping_sub(((i + 1).wrapping_sub(top) >> (usize::BITS - 1)) as limb_t);

        /*
         * Conditionally move current array by `1`, if not all needed bytes already read.
         *
         * This is done by applying bitwise and (&) on `1` and `mask`. Because mask is `0` when `i + 1` is >= `top`,
         * doing bitwise and will result in `0`, so slice will not be moved. Otherwise, mask will be `limb_t::MAX`, and
         * slice will be moved by `1`.
         */
        d = &d[(1 & mask).try_into().unwrap()..];
    }

    // Because offset won't always be divisible by `8`, we need to ignore remaining bits.
    ret >> (off % 8)
}

#[inline(always)]
unsafe fn vec_is_zero(a: *const byte, num: usize) -> limb_t {
    let ap = a as *const limb_t;
    let num = num / size_of::<limb_t>();

    let mut acc: limb_t = 0;
    for i in 0..num {
        acc |= *ap.wrapping_add(i);
    }

    is_zero(acc)
}

#[inline(always)]
unsafe fn vec_copy(ret: *mut u8, a: *const u8, num: usize) {
    let rp = ret as *mut limb_t;
    let ap = a as *const limb_t;

    let num = num / size_of::<limb_t>();

    for i in 0..num {
        *rp.wrapping_add(i) = *ap.wrapping_add(i);
    }
}

const BLS12_381_RX_P: blst_fp = blst_fp {
    l: [
        8505329371266088957,
        17002214543764226050,
        6865905132761471162,
        8632934651105793861,
        6631298214892334189,
        1582556514881692819,
    ],
};

unsafe fn p1_dadd_affine(
    p3: *mut P1XYZZ,
    p1: *const P1XYZZ,
    p2: *const blst_p1_affine,
    subtract: limb_t,
) {
    if vec_is_zero(p2 as *const u8, size_of::<blst_p1_affine>()) != 0 {
        vec_copy(p3 as *mut u8, p1 as *const u8, size_of::<P1XYZZ>());
        return;
    } else if vec_is_zero(
        &(*p1).zzz as *const blst_fp as *const u8,
        2 * size_of::<blst_fp>(),
    ) != 0
    {
        vec_copy(
            &mut ((*p3).x) as *mut blst_fp as *mut u8,
            &((*p2).x) as *const blst_fp as *const u8,
            2 * size_of::<blst_fp>(),
        );
        blst::blst_fp_cneg(&mut (*p3).zzz, &BLS12_381_RX_P, subtract != 0);
        vec_copy(
            &mut ((*p3).zz) as *mut blst_fp as *mut u8,
            &BLS12_381_RX_P as *const blst_fp as *const u8,
            size_of::<blst_fp>(),
        );
        return;
    }

    let mut p = blst_fp::default();
    let mut r = blst_fp::default();

    blst::blst_fp_mul(&mut p, &(*p2).x, &(*p1).zz);
    blst::blst_fp_mul(&mut r, &(*p2).y, &(*p1).zzz);
    blst::blst_fp_cneg(&mut r, &r, subtract != 0);
    blst::blst_fp_sub(&mut p, &p, &(*p1).x);
    blst::blst_fp_sub(&mut r, &r, &(*p1).y);

    if vec_is_zero(&p as *const blst_fp as *const u8, size_of::<blst_fp>()) == 0 {
        let mut pp = blst_fp::default();
        let mut ppp = blst_fp::default();
        let mut q = blst_fp::default();

        blst::blst_fp_sqr(&mut pp, &p);
        blst::blst_fp_mul(&mut ppp, &pp, &p);
        blst::blst_fp_mul(&mut q, &(*p1).x, &pp);
        blst::blst_fp_sqr(&mut (*p3).x, &r);
        blst::blst_fp_add(&mut p, &q, &q);
        blst::blst_fp_sub(&mut (*p3).x, &(*p3).x, &ppp);
        blst::blst_fp_sub(&mut (*p3).x, &(*p3).x, &p);
        blst::blst_fp_sub(&mut q, &q, &(*p3).x);
        blst::blst_fp_mul(&mut q, &q, &r);
        blst::blst_fp_mul(&mut (*p3).y, &(*p1).y, &ppp);
        blst::blst_fp_sub(&mut (*p3).y, &q, &(*p3).y);
        blst::blst_fp_mul(&mut (*p3).zz, &(*p1).zz, &pp);
        blst::blst_fp_mul(&mut (*p3).zzz, &(*p1).zzz, &ppp);
    } else if vec_is_zero(&r as *const blst_fp as *const u8, size_of::<blst_fp>()) != 0 {
        let mut u = blst_fp::default();
        let mut s = blst_fp::default();
        let mut m = blst_fp::default();

        blst::blst_fp_add(&mut u, &(*p2).y, &(*p2).y);
        blst::blst_fp_sqr(&mut (*p3).zz, &u);
        blst::blst_fp_mul(&mut (*p3).zzz, &(*p3).zz, &u);
        blst::blst_fp_mul(&mut s, &(*p2).x, &(*p3).zz);
        blst::blst_fp_sqr(&mut m, &(*p2).x);
        blst::blst_fp_mul_by_3(&mut m, &m);
        blst::blst_fp_sqr(&mut (*p3).x, &m);
        blst::blst_fp_add(&mut u, &s, &s);
        blst::blst_fp_sub(&mut (*p3).x, &(*p3).x, &u);
        blst::blst_fp_mul(&mut (*p3).y, &(*p3).zzz, &(*p2).y);
        blst::blst_fp_sub(&mut s, &s, &(*p3).x);
        blst::blst_fp_mul(&mut s, &s, &m);
        blst::blst_fp_sub(&mut (*p3).y, &s, &(*p3).y);
        blst::blst_fp_cneg(&mut (*p3).zzz, &(*p3).zzz, subtract != 0);
    } else {
        vec_zero(
            &mut (*p3).zzz as *mut blst_fp as *mut u64,
            2 * size_of::<blst_fp>(),
        );
    }
}

unsafe fn p1_dadd(p3: *mut P1XYZZ, p1: *const P1XYZZ, p2: *const P1XYZZ) {
    if vec_is_zero(
        &(*p2).zzz as *const blst_fp as *const u8,
        2 * size_of::<blst_fp>(),
    ) != 0
    {
        vec_copy(p3 as *mut u8, p1 as *const u8, size_of::<P1XYZZ>());
        return;
    } else if vec_is_zero(
        &(*p1).zzz as *const blst_fp as *const u8,
        2 * size_of::<blst_fp>(),
    ) != 0
    {
        vec_copy(p3 as *mut u8, p2 as *mut u8, size_of::<P1XYZZ>());
        return;
    }

    let mut u = blst_fp::default();
    let mut s = blst_fp::default();
    let mut p = blst_fp::default();
    let mut r = blst_fp::default();

    blst::blst_fp_mul(&mut u, &(*p1).x, &(*p2).zz);
    blst::blst_fp_mul(&mut s, &(*p1).y, &(*p2).zzz);
    blst::blst_fp_mul(&mut p, &(*p2).x, &(*p1).zz);
    blst::blst_fp_mul(&mut r, &(*p2).y, &(*p1).zzz);
    blst::blst_fp_sub(&mut p, &p, &u);
    blst::blst_fp_sub(&mut r, &r, &s);

    if vec_is_zero(&p as *const blst_fp as *const u8, size_of::<blst_fp>()) == 0 {
        let mut pp = blst_fp::default();
        let mut ppp = blst_fp::default();
        let mut q = blst_fp::default();

        blst::blst_fp_sqr(&mut pp, &p);
        blst::blst_fp_mul(&mut ppp, &pp, &p);
        blst::blst_fp_mul(&mut q, &u, &pp);
        blst::blst_fp_sqr(&mut (*p3).x, &r);
        blst::blst_fp_add(&mut p, &q, &q);
        blst::blst_fp_sub(&mut (*p3).x, &(*p3).x, &ppp);
        blst::blst_fp_sub(&mut (*p3).x, &(*p3).x, &p);
        blst::blst_fp_sub(&mut q, &q, &(*p3).x);
        blst::blst_fp_mul(&mut q, &q, &r);
        blst::blst_fp_mul(&mut (*p3).y, &s, &ppp);
        blst::blst_fp_sub(&mut (*p3).y, &q, &(*p3).y);
        blst::blst_fp_mul(&mut (*p3).zz, &(*p1).zz, &(*p2).zz);
        blst::blst_fp_mul(&mut (*p3).zzz, &(*p1).zzz, &(*p2).zzz);
        blst::blst_fp_mul(&mut (*p3).zz, &(*p3).zz, &pp);
        blst::blst_fp_mul(&mut (*p3).zzz, &(*p3).zzz, &ppp);
    } else if vec_is_zero(&r as *const blst_fp as *const u8, size_of::<blst_fp>()) != 0 {
        let mut v = blst_fp::default();
        let mut w = blst_fp::default();
        let mut m = blst_fp::default();

        blst::blst_fp_add(&mut u, &(*p1).y, &(*p1).y);
        blst::blst_fp_sqr(&mut v, &u);
        blst::blst_fp_mul(&mut w, &v, &u);
        blst::blst_fp_mul(&mut s, &(*p1).x, &v);
        blst::blst_fp_sqr(&mut m, &(*p1).x);
        blst::blst_fp_mul_by_3(&mut m, &m);
        blst::blst_fp_sqr(&mut (*p3).x, &m);
        blst::blst_fp_add(&mut u, &s, &s);
        blst::blst_fp_sub(&mut (*p3).x, &(*p3).x, &u);
        blst::blst_fp_mul(&mut (*p3).y, &w, &(*p1).y);
        blst::blst_fp_sub(&mut s, &s, &(*p3).x);
        blst::blst_fp_mul(&mut s, &s, &m);
        blst::blst_fp_sub(&mut (*p3).y, &s, &(*p3).y);
        blst::blst_fp_mul(&mut (*p3).zz, &(*p1).zz, &v);
        blst::blst_fp_mul(&mut (*p3).zzz, &(*p1).zzz, &w);
    } else {
        vec_zero(
            &mut (*p3).zzz as *mut blst_fp as *mut limb_t,
            2 * size_of::<blst_fp>(),
        );
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct P1XYZZ {
    x: blst_fp,
    y: blst_fp,
    zzz: blst_fp,
    zz: blst_fp,
}

/// Move point to corresponding bucket
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
fn p1s_bucket(
    buckets: &mut [P1XYZZ],
    mut booth_idx: limb_t,
    wbits: usize,
    point: *const blst_p1_affine,
) {
    /*
     * Get the `wbits + 1` bit in booth index.
     * This is a sign bit: `0` means scalar is positive, `1` - negative
     */
    let booth_sign = (booth_idx >> wbits) & 1;

    /*
     * Normalize bucket index.
     *
     * `(1 << wbits) - 1` generates number, which has `wbits` ones at the end.
     * For example:
     *     `wbits = 3` -> 0b00000111 (7)
     *     `wbits = 4` -> 0b00001111 (15)
     *     `wbits = 5` -> 0b00011111 (31)
     * And so on.
     *
     * Applying bitwise and (&) on `booth_idx` with such mask, means "leave only `wbits` bits from the end, and set all others to zero"
     * For example:
     *     `booth_idx = 14`,  `wbits = 3` -> 0b00001110 & 0b00000111 = 0b00000110
     *     `booth_idx = 255`, `wbits = 4` -> 0b11111111 & 0b00001111 = 0b00001111
     *     `booth_idx = 253`, `wbits = 5` -> 0b11111101 & 0b00011111 = 0b00011101
     */
    booth_idx &= (1 << wbits) - 1;

    // Bucket with index zero is ignored, as all values that fall in it are multiplied by zero (P * 0 = 0, no need to compute that).
    if booth_idx != 0 {
        // This command moves all buckets to the right, as bucket 0 doesn't exist (P * 0 = 0, no need to save it).
        booth_idx -= 1;

        let booth_idx: usize = booth_idx.try_into().unwrap();

        /*
         * When:
         *     `booth_sign = 0` -> add point to bucket[booth_idx]
         *     `booth_sign = 1` -> subtract point from bucket[booth_idx]
         */
        unsafe {
            p1_dadd_affine(
                &mut buckets[booth_idx],
                &buckets[booth_idx],
                point,
                booth_sign,
            );
        };
    }
}

unsafe fn p1_to_jacobian(out: *mut blst_p1, input: *const P1XYZZ) {
    // POINTonE1 *out, const POINTonE1xyzz *in

    // blst::blst_p1_from_jacobian(out, in_)

    // mul_fp(out->X, in->X, in->ZZ);
    blst::blst_fp_mul(&mut (*out).x, &(*input).x, &(*input).zz);
    // mul_fp(out->Y, in->Y, in->ZZZ);
    blst::blst_fp_mul(&mut (*out).y, &(*input).y, &(*input).zzz);
    // vec_copy(out->Z, in->ZZ, sizeof(out->Z));
    vec_copy(
        &mut (*out).z as *mut blst_fp as *mut u8,
        &(*input).zz as *const blst_fp as *const u8,
        size_of::<blst_fp>(),
    );
}

/// Calculate bucket sum
///
/// This function multiplies point in each bucket by it's index. Then, it will sum all multiplication results and write
/// resulting point to the `out`.
///
/// This function also clears all buckets (sets all values in buckets to zero.)
///
/// ## Arguments
///
/// * out     - output where bucket sum must be written
/// * buckets - pointer to the beginning of the array of buckets
/// * wbits   - window size, aka exponent of q (q^window)
///  
fn p1_integrate_buckets(out: &mut blst_p1, buckets: &mut [P1XYZZ], wbits: usize) {
    // Calculate total amount of buckets
    let mut n = (1usize << wbits) - 1;

    // Resulting point
    let mut ret = buckets[n];
    // Accumulator
    let mut acc = buckets[n];

    // Clear last bucket
    buckets[n] = P1XYZZ::default();

    /*
     * Sum all buckets.
     *
     * Starting from the end, this loop adds point to accumulator, and then adds point to the result.
     * If the point is in the bucket `i`, then adding this point to the accumulator and adding accumulator `i` times
     * helps to avoid multiplication of point by `i`.
     *
     * Example:
     *
     * If we have 3 buckets with points [`p1`, `p2`, `p3`], and we need to calculate bucket sum, naive approach would be:
     * `S` = `p1` + 2 * `p2` + 3 * `p3` (which is `p1` + `p2` + `p2` + `p3` + `p3` + `p3` - 5 additions)
     * But using accumulator, it would be:
     *
     * ```rust
     * acc = p3;
     * ret = p3;
     * acc += p2;   // now acc contains `p2` + `p3`
     * ret += acc;  // now res contains `p2` + 2*`p3`
     * acc += p1;   // now acc contains `p1` + `p2` + `p3`
     * ret += acc;  // now res contains `p1` + 2*`p2` + 3*`p3`
     * ```
     *
     * 4 additions. So using accumulator, we saved 1 addition.
     */
    loop {
        if n == 0 {
            break;
        }
        n -= 1;

        // Add point to accumulator
        unsafe { p1_dadd(&mut acc, &acc, &buckets[n]) };
        // Add accumulator to result
        unsafe { p1_dadd(&mut ret, &ret, &acc) };
        // Clear bucket
        buckets[n] = P1XYZZ::default();
    }

    // Convert point from magical 4-coordinate system to Jacobian (normal)
    unsafe { p1_to_jacobian(out, &ret) };
}

#[allow(clippy::too_many_arguments)]
#[cfg(all(feature = "parallel", feature = "std"))]
pub fn pippenger_tile_pub(
    ret: &mut blst_p1,
    points: &[blst_p1_affine],
    npoints: usize,
    scalars: &[u8],
    nbits: usize,
    buckets: &mut [P1XYZZ],
    bit0: usize,
    window: usize,
) {
    let (wbits, cbits) = if bit0 + window > nbits {
        let wbits = nbits - bit0;
        (wbits, wbits + 1)
    } else {
        (window, window)
    };

    pippenger_tile(
        ret, points, npoints, scalars, nbits, buckets, bit0, wbits, cbits,
    );
}

#[allow(clippy::too_many_arguments)]
fn pippenger_tile(
    ret: &mut blst_p1,
    points: &[blst_p1_affine],
    mut npoints: usize,
    scalars: &[u8],
    nbits: usize,
    buckets: &mut [P1XYZZ],
    bit0: usize,
    wbits: usize,
    cbits: usize,
) {
    // Calculate number of bytes, to fit `nbits`. Basically, this is division by 8 with rounding up to nearest integer.
    let nbytes = (nbits + 7) / 8;

    // Get first scalar
    let scalar = &scalars[0..nbytes];

    // Get first point
    let point = &points[0];

    // Create mask, that contains `wbits` ones at the end.
    let wmask = ((1 as limb_t) << (wbits + 1)) - 1;

    /*
     * Check if `bit0` is zero. `z` is set to `1` when `bit0 = 0`, and `0` otherwise.
     *
     * The `z` flag is used to do a small trick -
     */
    let z = is_zero(bit0.try_into().unwrap());

    // Offset `bit0` by 1, if it is not equal to zero.
    let bit0 = bit0 - (z ^ 1) as usize;

    // Increase `wbits` by one, if `bit0` is not equal to zero.
    let wbits = wbits + (z ^ 1) as usize;

    // Calculate first window value (encoded bucket index)
    let wval = (get_wval_limb(scalar, bit0, wbits) << z) & wmask;
    let mut wval = booth_encode(wval, cbits);

    // Get second scalar
    let scalar = &scalars[nbytes..2 * nbytes];

    // Calculate second window value (encoded bucket index)
    let wnxt = (get_wval_limb(scalar, bit0, wbits) << z) & wmask;
    let mut wnxt = booth_encode(wnxt, cbits);

    // Move first point to corresponding bucket
    p1s_bucket(buckets, wval, cbits, point);

    // Last point will be calculated separately, so decrementing point count
    npoints -= 1;

    // Move points to buckets
    for i in 1..npoints {
        // Get current window value (encoded bucket index)
        wval = wnxt;

        // Get next scalar
        let scalar = &scalars[(i + 1) * nbytes..(i + 2) * nbytes];
        // Get next window value (encoded bucket index)
        wnxt = (get_wval_limb(scalar, bit0, wbits) << z) & wmask;
        wnxt = booth_encode(wnxt, cbits);

        // TODO: add prefetching
        // POINTonE1_prefetch(buckets, wnxt, cbits);
        // p1_prefetch(buckets, wnxt, cbits);

        // Get current point
        let point = &points[i];

        // Move point to corresponding bucket (add or subtract from bucket)
        // `wval` contains encoded bucket index, as well as sign, which shows if point should be subtracted or added to bucket
        p1s_bucket(buckets, wval, cbits, point);
    }
    // Get last point
    let point = &points[npoints];
    // Move point to bucket
    p1s_bucket(buckets, wnxt, cbits, point);
    // Integrate buckets - multiply point in each bucket by scalar and sum all results
    p1_integrate_buckets(ret, buckets, cbits - 1);
}

fn pippenger_impl(
    points: &[blst_p1_affine],
    scalars: &[u8],
    nbits: usize,
    buckets: &mut [P1XYZZ],
    window: usize,
) -> blst_p1 {
    let mut ret = blst_p1::default();

    let mut wbits: usize = nbits % window;
    let mut cbits: usize = wbits + 1;
    let mut bit0: usize = nbits;
    let mut tile = blst_p1::default();

    loop {
        bit0 -= wbits;
        if bit0 == 0 {
            break;
        }

        pippenger_tile(
            &mut tile,
            points,
            points.len(),
            scalars,
            nbits,
            buckets,
            bit0,
            wbits,
            cbits,
        );

        // add bucket sum (aka tile) to the return value
        unsafe { blst_p1_add(&mut ret, &ret, &tile) };
        // multiply return value by Q (2^`window`) - double point `window` times.
        for _ in 0..window {
            unsafe { blst_p1_double(&mut ret, &ret) };
        }
        cbits = window;
        wbits = window;
    }
    pippenger_tile(
        &mut tile,
        points,
        points.len(),
        scalars,
        nbits,
        buckets,
        0,
        wbits,
        cbits,
    );
    unsafe { blst_p1_add(&mut ret, &ret, &tile) };

    ret
}

/// Single-threaded Pippenger's algorithm implementation
fn pippenger_sync(points: &[FsG1], scalars: &[FsFr]) -> FsG1 {
    // Convert points to affine
    let points = points_to_affine(points);

    if points.len() * size_of::<blst_p1_affine>() * 8 * 3 <= (144 * 1024) {
        let mut table = vec![blst_p1_affine::default(); points.len() * 8];

        let points_affine_arg: [*const blst_p1_affine; 2] = [points.as_ptr(), ptr::null()];

        unsafe {
            blst_p1s_mult_wbits_precompute(
                table.as_mut_ptr(),
                4,
                points_affine_arg.as_ptr(),
                points.len(),
            );
        };

        // Convert FsFr elements to blst_scalars
        let scalars = {
            let mut blst_scalars = vec![blst_scalar::default(); points.len()];

            for i in 0..points.len() {
                unsafe { blst_scalar_from_fr(&mut blst_scalars[i], &scalars[i].0) };
            }

            blst_scalars
        };

        let scalars_arg: [*const blst_scalar; 2] = [scalars.as_ptr(), ptr::null()];

        let mut ret = FsG1::default();

        unsafe {
            blst_p1s_mult_wbits(
                &mut ret.0,
                table.as_ptr(),
                4,
                points.len(),
                scalars_arg.as_ptr() as *const *const u8,
                255,
                ptr::null_mut(),
            );
        };

        ret
    } else {
        let window = pippenger_window_size(points.len());

        let mut buckets = vec![
            P1XYZZ::default();
            unsafe { blst_p1s_mult_wbits_scratch_sizeof(points.len()) }
                / size_of::<P1XYZZ>()
        ];

        // Convert FsFr to bytes
        let scalars = scalars_to_bytes(scalars);

        FsG1(pippenger_impl(&points, &scalars, 255, &mut buckets, window))
    }
}

/// Multithread Pippenger's algorithm implementation
#[cfg(all(feature = "parallel", feature = "std"))]
fn pippenger_async(points: &[FsG1], scalars: &[FsFr]) -> FsG1 {
    use crate::msm::pippenger_parallel::{da_pool, multiply};

    let pool = da_pool();
    let ncpus = pool.max_count();

    if ncpus < 2 || points.len() < 32 {
        return pippenger_sync(points, scalars);
    }

    let points = points_to_affine(points);
    let scalars = scalars_to_bytes(scalars);

    FsG1(multiply(
        &points,
        &scalars,
        255,
        pippenger_window_size(points.len()),
        pool,
    ))
}

/// Convert slice of points to blst_p1_affine's. It also respects parallel & no_std features
fn points_to_affine(points: &[FsG1]) -> Vec<blst_p1_affine> {
    // Parallel version is available, so use it
    #[cfg(all(feature = "parallel", feature = "std"))]
    {
        crate::msm::pippenger_parallel::points_to_affine(points)
    }

    // Parallelism is not available, do standard conversion
    #[cfg(not(all(feature = "parallel", feature = "std")))]
    {
        let mut points_affine = vec![blst_p1_affine::default(); points.len()];

        let points_arg: [*const blst_p1; 2] = [&points[0].0, ptr::null()];
        unsafe {
            blst::blst_p1s_to_affine(
                points_affine.as_mut_ptr(),
                points_arg.as_ptr(),
                points.len(),
            )
        };

        points_affine
    }
}

/// Function that converts scalars
/// TODO: add parallelism, if available
fn scalars_to_bytes(scalars: &[FsFr]) -> Vec<u8> {
    let mut blst_scalars = Vec::with_capacity(scalars.len() * 32);

    for el in scalars.iter() {
        let mut scalar = blst_scalar::default();

        unsafe { blst_scalar_from_fr(&mut scalar, &el.0) };

        blst_scalars.extend_from_slice(&scalar.b);
    }

    blst_scalars
}

pub fn pippenger(points: &[FsG1], scalars: &[FsFr]) -> FsG1 {
    #[cfg(all(feature = "parallel", feature = "std"))]
    return pippenger_async(points, scalars);

    #[cfg(not(all(feature = "parallel", feature = "std")))]
    return pippenger_sync(points, scalars);
}

#[cfg(test)]
mod tests {
    use crate::msm::pippenger::{booth_encode, get_wval_limb};

    #[test]
    fn booth_encode_must_produce_correct_results() {
        assert_eq!(booth_encode(0, 1), 0);
        assert_eq!(booth_encode(0, 5), 0);
        assert_eq!(booth_encode(1, 1), 1);
        assert_eq!(booth_encode(55, 5), 18446744073709551588);
    }

    #[test]
    fn get_wval_limb_example_1() {
        let val = get_wval_limb(&[0b01010111u8], 0, 4);
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

        //                     this is [128u8, 15u8, 3u8, 253u8] written in binary
        let scalar = [0b10000000, 0b00001111u8, 0b00000011u8, 0b11111101u8];
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
