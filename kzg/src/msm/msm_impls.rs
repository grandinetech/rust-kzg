// NOTES TO SELF:
// To run tests locally that check msm validity, run:
// cargo test --manifest-path blst/Cargo.toml --no-fail-fast --release --features c_bindings
// To run benchmarks locally, run
// cargo bench --manifest-path blst/Cargo.toml --features c_bindings

// Source: https://www.bmoeller.de/pdf/TI-01-08.multiexp.pdf
// according to this https://decentralizedthoughts.github.io/2025-02-14-verifiable-MSM/
// this algorithm can be applied for msms, as elliptic curves are
// cyclic additive groups, and joint multiexponentiation works in that case works
// the same as multi scalar multiplication
// and multiplication and squaring translates into
// addition and squaring

// https://doc-internal.dalek.rs/src/curve25519_dalek/backend/serial/scalar_mul/straus.rs.html#48-143
// https://www.jstor.org/stable/2310929?seq=2

use crate::{Fr, G1Affine, G1Fp, G1GetFp, G1Mul, G1ProjAddAffine, G1};
use alloc::vec::Vec;

#[cfg(all(feature = "arkmsm", not(feature = "parallel")))]
use super::arkmsm::arkmsm_msm::VariableBaseMSM;
use super::precompute::PrecomputationTable;

use super::tiling_pippenger_ops::tiling_pippenger;

#[cfg(feature = "parallel")]
use super::tiling_parallel_pippenger::{parallel_affine_conv, tiling_parallel_pippenger};

#[cfg(feature = "parallel")]
fn msm_parallel<
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
>(

    points: &[TG1],
    scalars: &[TFr],
    precomputation: Option<&PrecomputationTable<TFr, TG1, TG1Fp, TG1Affine, TProjAddAffine>>,
) -> TG1 {
    if let Some(precomputation) = precomputation {
        precomputation.multiply_parallel(scalars)
    } else {
        let (points, scalars): (Vec<_>, Vec<_>) = points
            .iter()
            .cloned()
            .zip(scalars.iter())
            .filter(|(p, _)| !p.is_inf())
            .collect();
        let points = batch_convert::<TG1, TG1Fp, TG1Affine>(&points);
        let scalars = scalars.iter().map(|s| s.to_scalar()).collect::<Vec<_>>();
        tiling_parallel_pippenger(&points, &scalars)
    }
}

pub fn pippenger<
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
>(
    points: &[TG1],
    scalars: &[TFr],
) -> TG1 {
    let (points, scalars): (Vec<_>, Vec<_>) = points
        .iter()
        .cloned()
        .zip(scalars.iter())
        .filter(|(p, _)| !p.is_inf())
        .collect();

    let points = batch_convert::<TG1, TG1Fp, TG1Affine>(&points);
    let scalars = scalars.iter().map(|s| s.to_scalar()).collect::<Vec<_>>();

    tiling_pippenger(&points, &scalars)
}

#[cfg(not(feature = "parallel"))]
#[allow(clippy::extra_unused_type_parameters)]
#[allow(unused_variables)]
fn msm_sequential<
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
>(

    points: &[TG1],
    scalars: &[TFr],
    precomputation: Option<&PrecomputationTable<TFr, TG1, TG1Fp, TG1Affine, TProjAddAffine>>,
) -> TG1 {
    #[cfg(not(feature = "arkmsm"))]
    {
        assert!(core::cmp::min(points.len(), scalars.len()) > 1);
        if let Some(precomputation) = precomputation {
            precomputation.multiply_sequential(scalars)
        } else {
            pippenger::<TFr, TG1, TG1Fp, TG1Affine, TProjAddAffine>(points, scalars)
        }
    }

    #[cfg(feature = "arkmsm")]
    {
        let (points, scalars): (Vec<_>, Vec<_>) = points
            .iter()
            .cloned()
            .zip(scalars.iter())
            .filter(|(p, _)| !p.is_inf())
            .collect();
        let points = batch_convert::<TG1, TG1Fp, TG1Affine>(&points);
        let scalars = scalars.iter().map(|s| s.to_scalar()).collect::<Vec<_>>();
        VariableBaseMSM::multi_scalar_mul::<TG1, TG1Fp, TG1Affine, TProjAddAffine>(
            &points, &scalars,
        )
    }
}

pub fn batch_convert<TG1: G1, TFp: G1Fp, TG1Affine: G1Affine<TG1, TFp> + Sized>(
    points: &[TG1],
) -> Vec<TG1Affine> {
    #[cfg(feature = "parallel")]
    return parallel_affine_conv::<TG1, TFp, TG1Affine>(points);

    #[cfg(not(feature = "parallel"))]
    return TG1Affine::into_affines(points);
}

/* 
 Straus (joint) unwindowed multi-scalar multiplication — explanation

 Idea:
 - Given n points P0..P{n-1} and n scalars s0..s{n-1}, compute
     R = sum_i si * Pi
   by processing all scalars together bit-by-bit from most significant bit (MSB)
   down to least significant bit (LSB).
 - Precompute a table of all 2^n possible combinations of the input points:
     table[mask] = sum_{i where bit i of mask is 1} Pi
   so each table entry is the group element corresponding to selecting a subset of points.
 - For each bit position b from MSB..0:
     - Double the accumulator R (R := 2*R)
     - Build a mask where bit i is the value of bit b of scalar si
     - If mask != 0, add table[mask] to R
 - The table lookup replaces up to n independent conditional additions per bit with a single
   table addition, which is efficient when n is small (typical use here: n < 8).
 - This is sometimes called joint sparse form / Straus multiexponentiation and is
   memory/time efficient for small n, since table size = 2^n.

 Complexity:
 - Precomputation: O(2^n) group additions (building table incrementally using lowest-bit trick).
 - Main loop: number_of_bits * (1 double + (1 table addition if mask != 0))
   plus cost to extract bits from scalars.
 - Best when n is small (e.g., n <= 7) because table size grows exponentially.

 Pseudocode:
   table[0] = 0
   for mask in 1..(1<<n)-1:
       lb = lowest_set_bit(mask)
       table[mask] = table[mask ^ (1<<lb)] + P[lb]

   R = 0
   for b in MSB..0:
       R = 2 * R
       mask = 0
       for i in 0..n-1:
           if bit_b(scalar_i) == 1:
               mask |= 1 << i
       if mask != 0:
           R = R + table[mask]

 Notes:
 - This implementation uses fixed-size scalar limbs (Scalar256::data) to find the MSB
   and to extract bits per scalar.
 - Works well for small n because it trades per-scalar conditional additions for a single
   indexed table addition per bit.
*/
fn straus_unwindowed<
    TG1: G1 + G1GetFp<TG1Fp> + G1Mul<TFr>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TFr: Fr,
>(

    points: &[TG1],
    scalars: &[TFr],
    len: usize,
) -> TG1 {
    // Convert to Scalar256 for bit access
    let mut svals: Vec<crate::Scalar256> = Vec::with_capacity(len);
    for i in 0..len {
        svals.push(scalars[i].to_scalar());
    }

    // window size is hardcoded 4 - we use 256 bit scalars, which are divided into 4 u64 type data units 

    // Find highest set bit among all scalars
    let mut max_bit: isize = -1;
    for s in &svals {
        // use length of data to stay generic (e.g. [u64; 4])
        let limbs = s.data.len();
        for limb in (0..limbs).rev() {
            let v = s.data[limb];
            if v != 0 {
                let leading = 63 - v.leading_zeros() as usize;
                let bit = (limb * 64 + leading) as isize; // 64 bits is used for the Scalar256
                if bit > max_bit {
                    max_bit = bit;
                }
                break;
            }
        }
    }

    if max_bit < 0 {
        return TG1::zero();
    }

    //to fail the test
    //return TG1::zero();

    // Precompute table of size 2^n (n = len) (all combinations)
    // 0 bit means excluded from some, 1 means included
    // 000
    // 001
    // ...
    // 111
    let n = len;
    let table_size = 1usize.checked_shl(n as u32).expect("n too large for table");
    let mut table: Vec<TG1> = Vec::with_capacity(table_size);
    table.push(TG1::zero()); // table[0] = 0
    for mask in 1..table_size {
        let lb = mask.trailing_zeros() as usize; // index of lowest set bit
        let prev = table[mask ^ (1 << lb)].clone(); // toggles the element presence at the index
        let mut cur = prev;
        // add point[lb]
        cur.add_or_dbl_assign(&points[lb]);
        table.push(cur);
    }

    // Main loop: from max_bit down to 0
    let mut out = TG1::zero();
    let max_b = max_bit as usize;
    for b in (0..=max_b).rev() {
        out.dbl_assign(); // double the accumulator
        // build mask for this bit
        let mut mask = 0usize;
        let limb_idx = b / 64; 
        

        let off = b % 64;
        for i in 0..n {
            // safety: if limb index is out of range treat as zero
            if limb_idx < svals[i].data.len() {
                if ((svals[i].data[limb_idx] >> off) & 1u64) == 1u64 {
                    mask |= 1 << i;
                }
            }
            // builds the required mask from the scalar, like
            // 1011, meaning P4 + P2 + P1 
        }


        if mask != 0 {
            // assignment to result, which will be double on the next iteration
            out.add_or_dbl_assign(&table[mask]);
        }
    }

    out
}

#[allow(clippy::extra_unused_type_parameters)]
pub fn msm<
    TG1: G1 + G1GetFp<TG1Fp> + G1Mul<TFr>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
    TFr: Fr,
>(

    points: &[TG1],
    scalars: &[TFr],
    len: usize,
    precomputation: Option<&PrecomputationTable<TFr, TG1, TG1Fp, TG1Affine, TProjAddAffine>>,
) -> TG1 {
    if len < 7 {
        return straus_unwindowed::<TG1, TG1Fp, TG1Affine, TFr>(points, scalars, len);
    }
    if len == 7 {
        let mut out = TG1::zero();
        for i in 0..len {
            let tmp = points[i].mul(&scalars[i]);
            out.add_or_dbl_assign(&tmp);
        }
        return out;
    }

    #[cfg(feature = "parallel")]
    return msm_parallel::<TFr, TG1, TG1Fp, TG1Affine, TProjAddAffine>(
        &points[0..len],
        &scalars[0..len],
        precomputation,
    );

    #[cfg(not(feature = "parallel"))]
    return msm_sequential::<TFr, TG1, TG1Fp, TG1Affine, TProjAddAffine>(
        &points[0..len],
        &scalars[0..len],
        precomputation,
    );
}
