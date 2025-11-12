// Source: https://www.bmoeller.de/pdf/TI-01-08.multiexp.pdf
// according to this https://decentralizedthoughts.github.io/2025-02-14-verifiable-MSM/
// this algorithm can be applied for msms, as elliptic curves are
// cyclic additive groups, and joint multiexponentiation works in that case works
// the same as multi scalar multiplication
// and multiplication and squaring translates into
// addition and squaring

// https://doc-internal.dalek.rs/src/curve25519_dalek/backend/serial/scalar_mul/straus.rs.html#48-143
// https://www.jstor.org/stable/2310929?seq=2
// tests
// NPOW=7 cargo +nightly fuzz run --features strauss blst_fixed_msm_with_zeros
// NPOW=7 cargo +nightly fuzz run --features strauss blst_fixed_msm
// strauss only
// cargo bench -p msm-benches --features strauss -- "straus msm mult"
// strauss compared to simple msm
// BENCH_NPOW=4 cargo bench -p msm-benches --features strauss -- "rust-kzg-blst"

// EXAMPLE (with CHUNK_SIZE=7):
// NPOW=7 cargo +nightly fuzz run --features strauss blst_fixed_msm
// This has 2^7=128 points, so it splits into 18.28 chunks of 7 points each
// Chunk 0:  points[0..7]     (7 points) → table size 2^7 = 128 entries
// Chunk 1:  points[7..14]    (7 points) → table size 2^7 = 128 entries
// Chunk 2:  points[14..21]   (7 points) → table size 2^7 = 128 entries
// ...
// Chunk 17: points[119..126] (7 points) → table size 2^7 = 128 entries
// Chunk 18: points[126..128] (2 points) → table size 2^2 = 4 entries
// This way, the maximum table size is limited to 128 entries, and
// doesn't eat up all the memory with 4 billion table entries
// result = chunk_0_result + chunk_1_result + ... + chunk_18_result
use alloc::vec::Vec;
use crate::{Fr, G1, G1Affine, G1Fp, G1GetFp, G1Mul, G1ProjAddAffine};
use core::marker::PhantomData;

/// Strauss chunk size: process this many points at a time.
/// Table size = 2^CHUNK_SIZE. For CHUNK_SIZE=6: table has 64 entries.
const STRAUSS_CHUNK_SIZE: usize = 7;

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

/// Algorithm:
/// Precompute table[mask] = sum of points indicated by bits in mask
///  mask=0b0000: identity (no points)
///  mask=0b0001: point[0]
///  mask=0b0011: point[0] + point[1]
///  mask=0b1111: point[0] + point[1] + point[2] + point[3]
///  Total entries: 2^len
/// 
/// 2. Process scalar bits from MSB to LSB:
///    - Double accumulator (left shift)
///    - Extract bit b from all scalars, form mask
///    - Add table[mask] to accumulator
/// 
/// Example for len=3, scalars=[s0, s1, s2]:
///   - If bit b: s0 has 1, s1 has 0, s2 has 1 → mask = 0b101 = 5
///   - Add table[5] = points[0] + points[2]
///
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

    
    if len == 0 {
        return TG1::zero();
    }

    let mut scalar_bits = Vec::with_capacity(len);
    let mut max_bit = 0usize;
    
    for scalar in scalars {
        let s = scalar.to_scalar();
        
        // Find highest set bit across all 64-bit limbs
        for (limb_idx, &limb) in s.data.iter().enumerate().rev() {
            if limb != 0 {
                let bit_pos = 63 - limb.leading_zeros() as usize;
                let global_bit = limb_idx * 64 + bit_pos;
                if global_bit > max_bit {
                    max_bit = global_bit;
                }
                break;
            }
        }
        scalar_bits.push(s);
    }

    // Build precomputation table of all 2^len point combinations
    // This is the expensive part that makes large len infeasible
    let table_size = 1 << len;  // 2^len
    let mut table: Vec<TG1> = Vec::with_capacity(table_size);
    table.push(TG1::zero());  // table[0] = identity

    // Incrementally build table using the "lowest bit trick"
    // For any mask, table[mask] = table[mask without lowest bit] + points[lowest bit position]
    // Example for len=3:
    //   table[0b000] = 0
    //   table[0b001] = 0 + P0 = P0
    //   table[0b010] = 0 + P1 = P1
    //   table[0b011] = P0 + P1
    //   table[0b100] = 0 + P2 = P2
    //   table[0b101] = P0 + P2
    //   table[0b110] = P1 + P2
    //   table[0b111] = P0 + P1 + P2
    for mask in 1..table_size {
        let lb = mask.trailing_zeros() as usize;  // Index of lowest set bit
        let prev_mask = mask ^ (1 << lb);          // Remove lowest bit
        
        if prev_mask == 0 {
            // Only one bit set: just copy the point
            table.push(points[lb].clone());
        } else {
            // Add current point to previous combination
            let mut new_val = table[prev_mask].clone();
            new_val.add_or_dbl_assign(&points[lb]);
            table.push(new_val);
        }
    }

    // Precompute bit masks for efficiency (avoid repeated bit extraction)
    // bit_masks[i] tells us which scalars have bit i set
    let mut bit_masks = vec![0usize; max_bit + 1];
    for bit in 0..=max_bit {
        let limb_idx = bit / 64;
        let bit_in_limb = bit % 64;
        let mut mask = 0usize;
        
        for i in 0..len {
            if limb_idx < scalar_bits[i].data.len() {
                if (scalar_bits[i].data[limb_idx] >> bit_in_limb) & 1 != 0 {
                    mask |= 1 << i;  // Set bit i if scalar i has this bit
                }
            }
        }
        bit_masks[bit] = mask;
    }

    // Main computation loop: process bits from MSB to LSB
    let mut result = TG1::zero();
    for bit in (0..=max_bit).rev() {
        result.dbl_assign(); 
        
        let mask = bit_masks[bit];
        if mask != 0 {
            result.add_or_dbl_assign(&table[mask]);
        }
    }

    result
}

pub fn straus_chunked<
    TG1: G1 + G1GetFp<TG1Fp> + G1Mul<TFr>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TFr: Fr,
>(
    points: &[TG1],
    scalars: &[TFr],
) -> TG1 {
    let n = points.len();
    
    if n == 0 {
        return TG1::zero();
    }

    //For very small n, just use Strauss directly
    // This will run up to NPOW=2, 3 will get chunked already
    if n <= STRAUSS_CHUNK_SIZE {
        return straus_unwindowed::<TG1, TG1Fp, TG1Affine, TFr>(points, scalars, n);
    }

    // Split into chunks and accumulate results
    let mut accumulator = TG1::zero();
    let num_chunks = (n + STRAUSS_CHUNK_SIZE - 1) / STRAUSS_CHUNK_SIZE;

    for chunk_idx in 0..num_chunks {
        let start = chunk_idx * STRAUSS_CHUNK_SIZE;
        let end = core::cmp::min(start + STRAUSS_CHUNK_SIZE, n);
        let chunk_size = end - start;

        let partial = straus_unwindowed::<TG1, TG1Fp, TG1Affine, TFr>(
            &points[start..end],
            &scalars[start..end],
            chunk_size,
        );
        accumulator.add_or_dbl_assign(&partial);
    }

    accumulator
}

impl<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>
    StraussTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>
where
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp> + Clone,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp> + Clone,
    TG1ProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
{
    pub fn new(points: &[TG1], _matrix: &[Vec<TG1>]) -> Result<Option<Self>, String> {
        let points_affine = TG1Affine::into_affines(points);

        let table = StraussTable {
            points: points_affine,
            numpoints: points.len(),
            batch_numpoints: 0,
            batch_points: Vec::new(),
            g1_marker: PhantomData,
            g1_fp_marker: PhantomData,
            fr_marker: PhantomData,
            g1_affine_add_marker: PhantomData,
        };

        Ok(Some(table))
    }

    /// Automatically splits into chunks of STRAUSS_CHUNK_SIZE to avoid
    /// exponential memory usage.
    pub fn multiply_sequential(&self, scalars: &[TFr]) -> TG1 {
        let n = scalars.len();
        // FALLBACK, BUT SINCE IT CHUNKS, IT SHOULDN'T GET OUT OF MEMORY ANYMORE
        // if n > self.points.len() {
        //     #[cfg(debug_assertions)]
        //     eprintln!(
        //         "[Strauss MSM] Requested {} points but only {} available.",
        //         n, self.points.len()
        //     );

        //     // Fallback to simple MSM
        //     let mut acc = TG1::zero();
        //     for (s, p_aff) in scalars.iter().zip(self.points.iter().cycle().take(n)) {
        //         let tmp = p_aff.to_proj().mul(s);
        //         acc.add_or_dbl_assign(&tmp);
        //     }
        //     return acc;
        // }
        let points_proj: Vec<TG1> = self.points[..n]
            .iter()
            .map(|a| a.to_proj())
            .collect();
        
        straus_chunked::<TG1, TG1Fp, TG1Affine, TFr>(&points_proj, scalars)
    }

    pub fn multiply_batch(&self, scalars: &[Vec<TFr>]) -> Vec<TG1> {
        scalars.iter().map(|s| self.multiply_sequential(s)).collect()
    }

    #[cfg(feature = "parallel")]
    pub fn multiply_parallel(&self, scalars: &[TFr]) -> TG1 {
        self.multiply_sequential(scalars)
    }
}