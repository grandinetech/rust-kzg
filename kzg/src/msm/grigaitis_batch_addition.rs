//! This algorithm is taken from crate-crypto's [rust-eth-kzg], adapted and
//! optimized for our usecase.
//!
//! [rust-eth-kzg]: https://github.com/crate-crypto/rust-eth-kzg/blob/acf3a5e8062e9a67c184c77ebff9c55223f9e034/crates/cryptography/bls12_381/src/batch_addition.rs
use alloc::{vec, vec::Vec};

use crate::{
    msm::pippenger_utils::{p1_dadd_affine, P1XYZZ},
    G1Affine, G1Fp, G1ProjAddAffine, G1,
};

/// This is the threshold to which batching the inversions in affine
/// formula costs more than doing mixed addition.
const BATCH_INVERSE_THRESHOLD: usize = 16;

/// Chooses between point addition and point doubling based on the input points.
#[inline(always)]
fn choose_add_or_double<TG1: G1, TG1Fp: G1Fp, TG1Affine: G1Affine<TG1, TG1Fp>>(
    pair: &mut [TG1Affine],
) -> TG1Fp {
    let fp = if pair[0].is_infinity() || pair[1].is_infinity() {
        return TG1Fp::one();
    } else if pair[0].x() == pair[1].x() {
        if pair[0].y() != pair[1].y() {
            pair[1] = TG1Affine::zero();
            pair[0] = TG1Affine::zero();
            return TG1Fp::one();
        }

        let f = pair[1].y().double();
        *pair[1].y_mut() = pair[0].x().square().mul3();
        f
    } else {
        *pair[1].y_mut() = pair[1].y().sub_fp(pair[0].y());
        pair[1].x().sub_fp(pair[0].x())
    };

    fp
}

/// Adds two elliptic curve points using the point addition/doubling formula.
///
/// Note: The inversion is precomputed and passed as a parameter.
///
/// This function handles both addition of distinct points and point doubling.
#[inline(always)]
fn point_add_double<TG1: G1, TG1Fp: G1Fp, TG1Affine: G1Affine<TG1, TG1Fp>>(
    p1: TG1Affine,
    p2: TG1Affine,
    inv: &TG1Fp,
) -> TG1Affine {
    if p1.is_zero() {
        return p2;
    }

    if p2.is_zero() {
        return p1;
    }

    let lambda = p2.y().mul_fp(inv);

    let x = lambda.square().sub_fp(p1.x()).sub_fp(p2.x());
    let y = lambda.mul_fp(&p1.x().sub_fp(&x)).sub_fp(p1.y());

    TG1Affine::from_xy(x, y)
}

/// Given a vector of field elements {v_i}, compute the vector {v_i^(-1)}
///
/// A scratchpad is used to avoid excessive allocations in the case that this method is
/// called repeatedly.
///
/// Panics if any of the elements are zero
pub fn batch_inverse_scratch_pad<F: G1Fp>(v: &mut [F], scratchpad: &mut Vec<F>) {
    if v.is_empty() {
        return;
    }

    // Montgomery's Trick and Fast Implementation of Masked AES
    // Genelle, Prouff and Quisquater
    // Section 3.2
    // but with an optimization to multiply every element in the returned vector by coeff

    // Clear the scratchpad and ensure it has enough capacity
    scratchpad.clear();
    scratchpad.reserve(v.len());

    // First pass: compute [a, ab, abc, ...]
    let mut tmp = F::one();
    for f in v.iter() {
        tmp = tmp.mul_fp(f);
        scratchpad.push(tmp);
    }

    // Invert `tmp`.
    tmp = tmp
        .inverse()
        .expect("guaranteed to be non-zero since we filtered out zero field elements");

    // Second pass: iterate backwards to compute inverses
    for i in (1..v.len()).rev() {
        // tmp := tmp * v[i]; v[i] := tmp * scratchpad[i - 1] = 1 / v[i]
        let new_tmp = tmp.mul_fp(&v[i]);
        v[i] = tmp.mul_fp(&scratchpad[i - 1]);
        tmp = new_tmp;
    }

    v[0] = tmp;
}

/// Performs multi-batch addition of multiple sets of elliptic curve points.
///
/// This function efficiently adds multiple sets of points amortizing the cost of the
/// inversion over all of the sets, using the same binary tree approach with striding
/// as the single-batch version.
pub fn multi_batch_addition_binary_tree_stride<
    TG1: G1,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TG1ProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
>(
    mut multi_points: Vec<Vec<TG1Affine>>,
) -> Vec<P1XYZZ<TG1Fp>> {
    let mut total_num_points = 0;
    let mut total_amount_of_work = 0;

    for points in &multi_points {
        total_num_points += points.len();
        total_amount_of_work += points.len() / 2;
    }

    let mut scratchpad = Vec::with_capacity(total_num_points);
    let mut denominators = Vec::with_capacity(total_num_points / 2);
    let mut sums = vec![P1XYZZ::<TG1Fp>::default(); multi_points.len()];

    // TODO: total_amount_of_work does not seem to be changing performance that much
    while total_amount_of_work > BATCH_INVERSE_THRESHOLD {
        total_amount_of_work = 0;
        denominators.clear();

        // For each pair of points over all
        // vectors, we collect them and put them in the
        // inverse array
        for (points, sum) in multi_points.iter_mut().zip(sums.iter_mut()) {
            // Make the number of points even
            if points.len() % 2 != 0 {
                p1_dadd_affine(sum, &points.pop().unwrap(), false);
            }
            if points.len() < 2 {
                continue;
            }

            for pair in points.chunks_exact_mut(2) {
                denominators.push(choose_add_or_double(pair));
            }
        }

        batch_inverse_scratch_pad(&mut denominators, &mut scratchpad);

        let mut denominators_offset = 0;

        for points in multi_points.iter_mut() {
            if points.len() < 2 {
                continue;
            }

            for (i, inv) in (0..=points.len() - 2)
                .step_by(2)
                .zip(&denominators[denominators_offset..])
            {
                points[i / 2] = point_add_double(points[i], points[i + 1], inv);
            }

            let num_points = points.len() / 2;
            // The latter half of the vector is now unused,
            // all results are stored in the former half.
            points.truncate(num_points);
            denominators_offset += num_points;
            total_amount_of_work += num_points / 2;
        }
    }

    for (sum, points) in sums.iter_mut().zip(multi_points) {
        for point in points {
            p1_dadd_affine(sum, &point, false);
        }
    }

    sums
}

#[cfg(test)]
mod test {
    use crate::msm::grigaitis_batch_addition::BATCH_INVERSE_THRESHOLD;

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn batch_inverse_threshold_is_valid() {
        assert!(
            BATCH_INVERSE_THRESHOLD >= 2,
            "THRESHOLD cannot be below the number of points needed for group addition"
        );
    }
}
