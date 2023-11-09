//! Multiscalar multiplication implementation using pippenger algorithm.
// use dusk_bytes::Serializable;

// use alloc::vec::*;

use crate::kzg_types::{ZFr, ZG1};
use bls12_381::{G1Projective, Scalar};

#[cfg(feature = "std")]
pub fn divn(mut scalar: Scalar, mut n: u32) -> Scalar {
    if n >= 256 {
        return Scalar::from(0);
    }

    while n >= 64 {
        let mut t = 0;
        for i in scalar.0.iter_mut().rev() {
            core::mem::swap(&mut t, i);
        }
        n -= 64;
    }

    if n > 0 {
        let mut t = 0;
        for i in scalar.0.iter_mut().rev() {
            let t2 = *i << (64 - n);
            *i >>= n;
            *i |= t;
            t = t2;
        }
    }

    scalar
}

/// Performs a Variable Base Multiscalar Multiplication.
#[allow(clippy::needless_collect)]
pub fn msm_variable_base(points_zg1: &[ZG1], zfrscalars: &[ZFr]) -> G1Projective {
    let g1_projective_vec = ZG1::converter(points_zg1);
    let points = g1_projective_vec.as_slice();

    let scalars_vec = ZFr::converter(zfrscalars);
    let scalars = scalars_vec.as_slice();

    #[cfg(feature = "parallel")]
    use rayon::prelude::*;

    let c = if scalars.len() < 32 {
        3
    } else {
        ln_without_floats(scalars.len()) + 2
    };

    let num_bits = 255usize;
    let fr_one = Scalar::one();

    let zero = G1Projective::identity();
    let window_starts: Vec<_> = (0..num_bits).step_by(c).collect();

    #[cfg(feature = "parallel")]
    let window_starts_iter = window_starts.into_par_iter();
    #[cfg(not(feature = "parallel"))]
    let window_starts_iter = window_starts.into_iter();

    // Each window is of size `c`.
    // We divide up the bits 0..num_bits into windows of size `c`, and
    // in parallel process each such window.
    let window_sums: Vec<_> = window_starts_iter
        .map(|w_start| {
            let mut res = zero;
            // We don't need the "zero" bucket, so we only have 2^c - 1 buckets
            let mut buckets = vec![zero; (1 << c) - 1];
            scalars
                .iter()
                .zip(points)
                .filter(|(s, _)| !(*s == &Scalar::zero()))
                .for_each(|(&scalar, base)| {
                    if scalar == fr_one {
                        // We only process unit scalars once in the first window.
                        if w_start == 0 {
                            res = res.add(base);
                        }
                    } else {
                        let mut scalar = Scalar::montgomery_reduce(
                            scalar.0[0],
                            scalar.0[1],
                            scalar.0[2],
                            scalar.0[3],
                            0,
                            0,
                            0,
                            0,
                        );

                        // We right-shift by w_start, thus getting rid of the
                        // lower bits.
                        scalar = divn(scalar, w_start as u32);
                        // We mod the remaining bits by the window size.
                        let scalar = scalar.0[0] % (1 << c);

                        // If the scalar is non-zero, we update the corresponding
                        // bucket.
                        // (Recall that `buckets` doesn't have a zero bucket.)
                        if scalar != 0 {
                            buckets[(scalar - 1) as usize] =
                                buckets[(scalar - 1) as usize].add(base);
                        }
                    }
                });

            let mut running_sum = G1Projective::identity();
            for b in buckets.into_iter().rev() {
                running_sum += b;
                res += &running_sum;
            }

            res
        })
        .collect();

    // We store the sum for the lowest window.
    let lowest = *window_sums.first().unwrap();
    // We're traversing windows from high to low.
    window_sums[1..]
        .iter()
        .rev()
        .fold(zero, |mut total, sum_i| {
            total += sum_i;
            for _ in 0..c {
                total = total.double();
            }
            total
        })
        + lowest
}

fn ln_without_floats(a: usize) -> usize {
    // log2(a) * ln(2)
    (log2(a) * 69 / 100) as usize
}
fn log2(x: usize) -> u32 {
    if x <= 1 {
        return 0;
    }

    let n = x.leading_zeros();
    core::mem::size_of::<usize>() as u32 * 8 - n
}

/*

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn pippenger_test() {
        // Reuse points across different tests
        let mut n = 512;
        let x = Scalar::from(2128506u64).invert().unwrap();
        let y = Scalar::from(4443282u64).invert().unwrap();
        let points = (0..n)
            .map(|i| G1Projective::generator() * Scalar::from(1 + i as u64))
            .collect::<Vec<_>>();
        let scalars = (0..n)
            .map(|i| x + (Scalar::from(i as u64) * y))
            .collect::<Vec<_>>(); // fast way to make ~random but deterministic scalars
        let premultiplied: Vec<G1Projective> = scalars
            .iter()
            .zip(points.iter())
            .map(|(sc, pt)| pt * sc)
            .collect();
        while n > 0 {
            let scalars = &scalars[0..n];
            let points = &points[0..n];
            let control: G1Projective = premultiplied[0..n].iter().sum();
            let subject = pippenger(
                points.to_owned().into_iter(),
                scalars.to_owned().into_iter(),
            );
            assert_eq!(subject, control);
            n = n / 2;
        }
    }

    #[test]
    fn msm_variable_base_test() {
        let points = vec![G1Affine::generator()];
        let scalars = vec![Scalar::from(100u64)];
        let premultiplied = G1Projective::generator() * Scalar::from(100u64);
        let subject = msm_variable_base(&points, &scalars);
        assert_eq!(subject, premultiplied);
    }
}
*/
