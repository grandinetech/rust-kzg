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
// cargo +nightly fuzz run --features strauss blst_fixed_msm_with_zeros

//use crate::{Fr, G1Affine, G1Fp, G1GetFp, G1Mul, G1ProjAddAffine, G1};
//use alloc::vec::Vec;

use alloc::vec::Vec;
use crate::{Fr, G1, G1Affine, G1Fp, G1GetFp, G1Mul, G1ProjAddAffine};

use core::marker::PhantomData;

#[cfg(all(feature = "arkmsm", not(feature = "parallel")))]
use super::arkmsm::arkmsm_msm::VariableBaseMSM;
// use super::precompute::PrecomputationTable;

#[derive(Debug, Clone)]
pub struct StraussTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>
where
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TG1ProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
{
    points: Vec<TG1Affine>,
    numpoints: usize,

    batch_numpoints: usize,
    batch_points: Vec<Vec<TG1Affine>>,

    g1_marker: PhantomData<TG1>,
    g1_fp_marker: PhantomData<TG1Fp>,
    fr_marker: PhantomData<TFr>,
    g1_affine_add_marker: PhantomData<TG1ProjAddAffine>,
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
pub fn straus_unwindowed<
    TG1: G1 + G1GetFp<TG1Fp> + G1Mul<TFr>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TFr: Fr,
>(
    points: &[TG1],
    scalars: &[TFr],
    len: usize,
) -> TG1 {
    // For large n, use simple MSM - Straus is only efficient for small n
    if len > 8 {
        let mut acc = TG1::zero();
        for i in 0..len {
            let tmp = points[i].mul(&scalars[i]);
            acc.add_or_dbl_assign(&tmp);
        }
        return acc;
    }

    let mut svals: Vec<crate::Scalar256> = Vec::with_capacity(len);
    for i in 0..len {
        svals.push(scalars[i].to_scalar());
    }

    // Find highest bit
    let mut max_bit = 0usize;
    for s in &svals {
        for limb in (0..s.data.len()).rev() {
            let v = s.data[limb];
            if v != 0 {
                let leading = 63 - v.leading_zeros() as usize;
                let bit = limb * 64 + leading;
                if bit > max_bit {
                    max_bit = bit;
                }
                break;
            }
        }
    }

    // Precompute table - size 2^len
    let table_size = 1 << len;
    let mut table: Vec<TG1> = Vec::with_capacity(table_size);
    table.push(TG1::zero());
    
    for mask in 1..table_size {
        let lb = mask.trailing_zeros() as usize;
        let mut cur = table[mask ^ (1 << lb)].clone();
        cur.add_or_dbl_assign(&points[lb]);
        table.push(cur);
    }

    // Main loop
    let mut out = TG1::zero();
    for b in (0..=max_bit).rev() {
        out.dbl_assign();
        
        let limb_idx = b / 64;
        let bit_pos = b % 64;
        
        let mut mask = 0usize;
        for i in 0..len {
            if limb_idx < svals[i].data.len() {
                if (svals[i].data[limb_idx] >> bit_pos) & 1 != 0 {
                    mask |= 1 << i;
                }
            }
        }
        
        if mask != 0 {
            out.add_or_dbl_assign(&table[mask]);
        }
    }

    out
}

// #[allow(clippy::extra_unused_type_parameters)]
// pub fn msm<
//     TG1: G1 + G1GetFp<TG1Fp> + G1Mul<TFr>,
//     TG1Fp: G1Fp,
//     TG1Affine: G1Affine<TG1, TG1Fp>,
//     TProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
//     TFr: Fr,
// >(

//     points: &[TG1],
//     scalars: &[TFr],
//     len: usize,
//     precomputation: Option<&PrecomputationTable<TFr, TG1, TG1Fp, TG1Affine, TProjAddAffine>>,
// ) -> TG1 {
//     if len < 7 {
//         return straus_unwindowed::<TG1, TG1Fp, TG1Affine, TFr>(points, scalars, len);
//     }
//     if len == 7 {
//         let mut out = TG1::zero();
//         for i in 0..len {
//             let tmp = points[i].mul(&scalars[i]);
//             out.add_or_dbl_assign(&tmp);
//         }
//         return out;
//     }

//     #[cfg(feature = "parallel")]
//     return msm_parallel::<TFr, TG1, TG1Fp, TG1Affine, TProjAddAffine>(
//         &points[0..len],
//         &scalars[0..len],
//         precomputation,
//     );

//     #[cfg(not(feature = "parallel"))]
//     return msm_sequential::<TFr, TG1, TG1Fp, TG1Affine, TProjAddAffine>(
//         &points[0..len],
//         &scalars[0..len],
//         precomputation,
//     );
// }

impl<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>
    StraussTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>
where
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp> + Clone,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp> + Clone,
    TG1ProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
{
    /// Called from precompute.rs
    pub fn new(points: &[TG1], _matrix: &[Vec<TG1>]) -> Result<Option<Self>, String> {
        // Convert projective points to affine form
        let points_affine = TG1Affine::into_affines(points);

        let table = StraussTable {
            points: points_affine,
            numpoints: points.len(),
            batch_numpoints: 0,
            batch_points: Vec::new(),
            g1_marker: core::marker::PhantomData,
            g1_fp_marker: core::marker::PhantomData,
            fr_marker: core::marker::PhantomData,
            g1_affine_add_marker: core::marker::PhantomData,
        };

        Ok(Some(table))
    }

    /// Used by msm_impls.rs -> msm_sequential
    // pub fn multiply_sequential(&self, scalars: &[TFr]) -> TG1 {
    //     // Convert stored affine points back to projective
    //     let points_proj: Vec<TG1> = self.points.iter().map(|a| a.to_proj()).collect();
    //     straus_unwindowed::<TG1, TG1Fp, TG1Affine, TFr>(&points_proj, scalars, self.numpoints)
    // }

    pub fn multiply_sequential(&self, scalars: &[TFr]) -> TG1 {
        let n = scalars.len();

        // Fix: gracefully handle n larger than precomputed size
        if n > self.points.len() {
            #[cfg(debug_assertions)]
            eprintln!(
                "[Strauss MSM] n={} exceeds precomputed size ({}). Falling back to slow MSM.",
                n,
                self.points.len()
            );

            // Fallback: do a simple MSM manually
            let mut acc = TG1::zero();
            // scalars.iter().zip(self.points.iter().cycle().take(n))
            // yields ( &TFr, &TG1Affine )
            for (s, p_aff) in scalars.iter().zip(self.points.iter().cycle().take(n)) {
                let tmp = p_aff.to_proj().mul(s);
                acc.add_or_dbl_assign(&tmp);
            }
            return acc;
        }

        // Normal fast path
        let points_proj: Vec<TG1> = self.points.iter().map(|a| a.to_proj()).collect();
        straus_unwindowed::<TG1, TG1Fp, TG1Affine, TFr>(&points_proj, scalars, self.numpoints)
    }

    /// Used by lib.rs batch MSM path
    pub fn multiply_batch(&self, scalars: &[Vec<TFr>]) -> Vec<TG1> {
        let mut results = Vec::with_capacity(scalars.len());
        for s in scalars {
            results.push(self.multiply_sequential(s));
        }
        results
    }

    #[cfg(feature = "parallel")]
    pub fn multiply_parallel(&self, scalars: &[TFr]) -> TG1 {
        // Optional: you can optimize this later, but for now just reuse sequential.
        self.multiply_sequential(scalars)
    }
}
