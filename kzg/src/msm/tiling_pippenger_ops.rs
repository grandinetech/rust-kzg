use core::mem::size_of;

use crate::{G1Affine, G1Fp, G1GetFp, Scalar256, G1};

use alloc::vec;

const fn is_zero(val: u64) -> u64 {
    (!val & (val.wrapping_sub(1))) >> (u64::BITS - 1)
}

const fn booth_encode(wval: u64, sz: usize) -> u64 {
    let mask = 0u64.wrapping_sub(wval >> sz);

    let wval = (wval + 1) >> 1;
    (wval ^ mask).wrapping_sub(mask)
}

#[inline(always)]
fn vec_zero_rt(ret: *mut u64, mut num: usize) {
    num /= size_of::<usize>();
    for i in 0..num {
        unsafe {
            *ret.add(i) = 0;
        }
    }
}

#[inline(always)]
fn type_zero<T>(ret: &mut T) {
    let rp = ret as *mut T as *mut u64;
    let num = size_of::<T>() / size_of::<u64>();

    for i in 0..num {
        unsafe {
            *rp.wrapping_add(i) = 0;
        }
    }
}

fn get_wval_limb(d: &Scalar256, off: usize, bits: usize) -> u64 {
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

#[inline(always)]
fn vec_is_zero(a: *const u8, num: usize) -> u64 {
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
fn type_is_zero<T>(a: &T) -> u64 {
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

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct P1XYZZ<TFp: G1Fp> {
    x: TFp,
    y: TFp,
    zzz: TFp,
    zz: TFp,
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

        out.zzz = TFp::BLS12_381_RX_P;
        if subtract {
            out.zzz.neg_assign();
        }

        out.zz = TFp::BLS12_381_RX_P;
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

fn p1_dadd<TFp: G1Fp>(out: &mut P1XYZZ<TFp>, p2: &P1XYZZ<TFp>) {
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

fn p1s_bucket<TG1: G1, TFp: G1Fp, TG1Affine: G1Affine<TG1, TFp>>(
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

fn p1_to_jacobian<TG1: G1 + G1GetFp<TFp>, TFp: G1Fp>(out: &mut TG1, input: &P1XYZZ<TFp>) {
    *out.x_mut() = input.x.mul_fp(&input.zz);
    *out.y_mut() = input.y.mul_fp(&input.zzz);
    *out.z_mut() = input.zz;
}

fn p1_integrate_buckets<TG1: G1 + G1GetFp<TFp>, TFp: G1Fp>(
    out: &mut TG1,
    buckets: &mut [P1XYZZ<TFp>],
    wbits: usize,
) {
    let mut n = (1usize << wbits) - 1;
    let mut ret = buckets[n];
    let mut acc = buckets[n];

    type_zero(&mut buckets[n]);
    loop {
        if n == 0 {
            break;
        }
        n -= 1;

        if type_is_zero(&buckets[n]) == 0 {
            p1_dadd(&mut acc, &buckets[n]);
            type_zero(&mut buckets[n]);
        }
        p1_dadd(&mut ret, &acc);
    }

    p1_to_jacobian(out, &ret);
}

#[allow(clippy::too_many_arguments)]
pub fn p1s_tile_pippenger_pub<TG1: G1 + G1GetFp<TFp>, TFp: G1Fp, TG1Affine: G1Affine<TG1, TFp>>(
    ret: &mut TG1,
    points: &[TG1Affine],
    scalars: &[Scalar256],
    buckets: &mut [P1XYZZ<TFp>],
    bit0: usize,
    window: usize,
) {
    const NBITS: usize = 255;
    let (wbits, cbits) = if bit0 + window > NBITS {
        let wbits = NBITS - bit0;
        (wbits, wbits + 1)
    } else {
        (window, window)
    };

    p1s_tile_pippenger(ret, points, scalars, buckets, bit0, wbits, cbits);
}

#[allow(clippy::too_many_arguments)]
pub fn p1s_tile_pippenger<TG1: G1 + G1GetFp<TFp>, TFp: G1Fp, TG1Affine: G1Affine<TG1, TFp>>(
    ret: &mut TG1,
    points: &[TG1Affine],
    scalars: &[Scalar256],
    buckets: &mut [P1XYZZ<TFp>],
    bit0: usize,
    wbits: usize,
    cbits: usize,
) {
    // Get first scalar
    let scalar = &scalars[0];

    // Get first point
    let point = &points[0];

    // Create mask, that contains `wbits` ones at the end.
    let wmask = (1u64 << (wbits + 1)) - 1;

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
    let scalar = &scalars[1];

    // Calculate second window value (encoded bucket index)
    let wnxt = (get_wval_limb(scalar, bit0, wbits) << z) & wmask;
    let mut wnxt = booth_encode(wnxt, cbits);

    // Move first point to corresponding bucket
    p1s_bucket(buckets, wval, cbits, point);

    // Last point will be calculated separately, so decrementing point count
    let npoints = points.len() - 1;

    // Move points to buckets
    for i in 1..npoints {
        // Get current window value (encoded bucket index)
        wval = wnxt;

        // Get next scalar
        let scalar = &scalars[i + 1];
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

pub fn tiling_pippenger<TG1: G1 + G1GetFp<TG1Fp>, TG1Fp: G1Fp, TG1Affine: G1Affine<TG1, TG1Fp>>(
    points: &[TG1Affine],
    scalars: &[Scalar256],
) -> TG1 {
    let window = pippenger_window_size(points.len());
    let mut buckets = vec![P1XYZZ::<TG1Fp>::default(); 1 << (window - 1)];

    let mut wbits: usize = 255 % window;
    let mut cbits: usize = wbits + 1;
    let mut bit0: usize = 255;
    let mut tile = TG1::default();

    let mut ret = TG1::default();

    loop {
        bit0 -= wbits;
        if bit0 == 0 {
            break;
        }

        p1s_tile_pippenger(&mut tile, points, scalars, &mut buckets, bit0, wbits, cbits);

        ret.add_assign(&tile);
        for _ in 0..window {
            ret.dbl_assign();
        }
        cbits = window;
        wbits = window;
    }
    p1s_tile_pippenger(&mut tile, points, scalars, &mut buckets, 0, wbits, cbits);
    ret.add_assign(&tile);
    ret
}

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

pub const fn num_bits(l: usize) -> usize {
    8 * core::mem::size_of::<usize>() - l.leading_zeros() as usize
}
