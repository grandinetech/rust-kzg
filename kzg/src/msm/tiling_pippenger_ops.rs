use crate::{G1Affine, G1Fp, G1GetFp, Scalar256, G1};

use alloc::vec;

use super::pippenger_utils::{
    booth_decode, booth_encode, get_wval_limb, is_zero, p1_dadd, p1_to_jacobian,
    pippenger_window_size, type_is_zero, type_zero, P1XYZZ,
};

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
    booth_decode(buckets, wval, cbits, point);

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
        booth_decode(buckets, wval, cbits, point);
    }
    // Get last point
    let point = &points[npoints];
    // Move point to bucket
    booth_decode(buckets, wnxt, cbits, point);
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
