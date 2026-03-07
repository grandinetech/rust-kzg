use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::ops::Sub;

use crate::{Fr, G1Affine, G1Fp, G1Mul, G1ProjAddAffine, Scalar256, G1};

pub struct BosCosterMSM;

impl BosCosterMSM {
    // https://github.com/btclib-org/btclib was originally used as reference for this implementation
    pub fn multi_scalar_mul<
        TFr: Fr,
        TG1: G1 + G1Mul<TFr>,
        TG1Fp: G1Fp,
        TG1Affine: G1Affine<TG1, TG1Fp>,
        TProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
    >(
        points: &[TG1],
        scalars: &[Scalar256],
    ) -> TG1 {
        debug_assert!(points.len() == scalars.len());

        let mut work_points = Vec::with_capacity(points.len());
        let mut heap: BinaryHeap<Pair> = BinaryHeap::with_capacity(points.len());

        for (s, p) in scalars.iter().zip(points.iter()) {
            if !s.is_zero() && !p.is_inf() {
                let idx = work_points.len();
                work_points.push(p.clone());
                heap.push(Pair {
                    scalar: *s,
                    point_index: idx,
                });
            }
        }

        while heap.len() > 1 {
            let Some(pair1) = heap.pop() else {
                break;
            };
            let Some(pair2) = heap.peek() else {
                break;
            };
            // the optimization for large scalar differences was taken from: https://link.springer.com/article/10.1007/s13389-012-0027-1
            let (k, pair2_scalar_2powk) = max_pow2_multiple_leq(&pair1.scalar, &pair2.scalar);
            let mut new_point = work_points[pair1.point_index].clone();
            for _ in 0..k {
                new_point.dbl_assign();
            }

            work_points[pair2.point_index].add_or_dbl_assign(&new_point);

            let scalar = pair1.scalar.sub(pair2_scalar_2powk);
            if !scalar.is_zero() {
                heap.push(Pair {
                    scalar,
                    point_index: pair1.point_index,
                });
            }
        }

        if let Some(pair) = heap.pop() {
            work_points[pair.point_index].mul(&TFr::from_u64_arr(&pair.scalar.data))
        } else {
            TG1::zero()
        }
    }
}

// This function was generated with ChatGPT
#[inline(always)]
fn scalar256_shr_once_assign(input: &mut Scalar256) {
    let mut carry = 0u64;
    for limb in input.data.iter_mut().rev() {
        let new_carry = (*limb & 1) << 63;
        *limb = (*limb >> 1) | carry;
        carry = new_carry;
    }
}

struct Pair {
    scalar: Scalar256,
    point_index: usize,
}

impl PartialEq for Pair {
    fn eq(&self, other: &Self) -> bool {
        self.scalar == other.scalar
    }
}

impl Eq for Pair {}

impl Ord for Pair {
    fn cmp(&self, other: &Self) -> Ordering {
        self.scalar.cmp(&other.scalar)
    }
}

impl PartialOrd for Pair {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Computes the largest `k` such that `(y << k) <= x`.
///
/// Returns `(k, y * 2^k)`.
///
/// Assumes `x >= y` and both scalars are non-zero.
#[inline(always)]
fn max_pow2_multiple_leq(x: &Scalar256, y: &Scalar256) -> (u32, Scalar256) {
    debug_assert!(x >= y);
    let bitlen_x = x.leading_zeros();
    let bitlen_y = y.leading_zeros();
    if bitlen_x == bitlen_y {
        return (0, *y);
    }
    let mut d = bitlen_x - bitlen_y;
    let mut shifted_y = *y << d;
    if shifted_y > *x {
        d -= 1;
        scalar256_shr_once_assign(&mut shifted_y);
    }
    (d, shifted_y)
}
