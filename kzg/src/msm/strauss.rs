use alloc::vec::Vec;
use crate::{Fr, G1, G1Affine, G1Fp, G1GetFp, G1Mul, G1ProjAddAffine};
use core::marker::PhantomData;

#[cfg(feature = "file-io")]
use alloc::string::String;

#[cfg(feature = "file-io")]
use serde::{Serialize, Deserialize};

// Strauss chunk size: process this many points at a time.
// Table size = 2^CHUNK_SIZE. For CHUNK_SIZE=7: table has 128 entries.


#[derive(Debug, Clone)]
pub struct StraussTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>
where
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TG1ProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
{
    /// precomputed per-chunk tables (each chunk table is a Vec of affine points
    /// holding sums for each mask in 0..(1<<chunk_len))
    chunk_tables: Vec<Vec<TG1Affine>>,
    /// sizes for each chunk (so last chunk may be < STRAUSS_CHUNK_SIZE)
    chunk_sizes: Vec<usize>,

    numpoints: usize,

    batch_numpoints: usize,
    batch_points: Vec<Vec<TG1Affine>>,

    g1_marker: PhantomData<TG1>,
    g1_fp_marker: PhantomData<TG1Fp>,
    fr_marker: PhantomData<TFr>,
    g1_affine_add_marker: PhantomData<TG1ProjAddAffine>,
}

#[cfg(feature = "file-io")]
#[derive(Serialize, Deserialize)]
struct SerializableChunkTable<TG1AffineSer> {
    chunk_sizes: Vec<usize>,
    // we store each chunk as a flat vector; caller must ensure TG1AffineSer is serializable
    chunk_tables: Vec<Vec<TG1AffineSer>>,
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
    /// Build a StraussTable that precomputes chunk tables for all chunks.
    /// This mirrors the style of other algorithms that precompute and store tables.
    pub fn new(points: &[TG1], _matrix: &[Vec<TG1>]) -> Result<Option<Self>, String> {
        let n = points.len();
        if n == 0 {
            // return an empty table (consistent with other implementations returning Some)
            let table = StraussTable {
                chunk_tables: Vec::new(),
                chunk_sizes: Vec::new(),
                numpoints: 0,
                batch_numpoints: 0,
                batch_points: Vec::new(),
                g1_marker: PhantomData,
                g1_fp_marker: PhantomData,
                fr_marker: PhantomData,
                g1_affine_add_marker: PhantomData,
            };
            return Ok(Some(table));
        }

        // Convert all points to affine once
        let points_affine: Vec<TG1Affine> = TG1Affine::into_affines(points);

        // Split into chunks
        let mut chunk_tables: Vec<Vec<TG1Affine>> = Vec::new();
        let mut chunk_sizes: Vec<usize> = Vec::new();

        let STRAUSS_CHUNK_SIZE: usize = match std::env::var("WINDOW_SIZE") {
            Ok(s) => s
                .parse()
                .expect("WINDOW_SIZE environment variable must be a valid number"),
            Err(_) => 8usize,
        };

        let num_chunks = (n + STRAUSS_CHUNK_SIZE - 1) / STRAUSS_CHUNK_SIZE;

        for chunk_idx in 0..num_chunks {
            let start = chunk_idx * STRAUSS_CHUNK_SIZE;
            let end = core::cmp::min(start + STRAUSS_CHUNK_SIZE, n);
            let chunk_len = end - start;
            chunk_sizes.push(chunk_len);

            // size of table for this chunk: 2^chunk_len
            let table_size = 1usize << chunk_len;

            // preallocate table
            let mut table: Vec<TG1Affine> = Vec::with_capacity(table_size);
            // We will build table in-affine form:
            // table[0] = identity (as affine identity; we need a representation).
            // However, many affine types cannot represent identity directly; but existing
            // code used TG1::zero() in proj form. To keep consistent, we'll build entries
            // as affine sums by converting sums back to affine via to_affine()
            // Use TG1 (projective) as intermediate for additions.

            // push placeholder for table[0] as the affine of TG1::zero()
            // We need a way to obtain affine identity; if TG1Affine has a constructor from proj zero,
            // the safest is to compute TG1::zero().to_affine() if the API provides it.
            // If not available, we will populate table[0] later after computing entries.
            // For portability, push a clone of points_affine[start] as placeholder and overwrite entry 0 later.
            // But it's better to push the proper identity if possible:
            // We'll attempt to get affine identity via TG1::zero().to_affine()
            // If `to_affine` method not available we fall back to a sentinel (first point) and then fix below.
            let mut table_proj: Vec<TG1> = Vec::with_capacity(table_size);
            table_proj.push(TG1::zero()); // table_proj[0] = identity

            // Build incremental table using lowest-bit trick; we do this in projective space for easier adds.
            for mask in 1..table_size {
                let lb = mask.trailing_zeros() as usize;
                let prev = mask ^ (1 << lb);
                if prev == 0 {
                    // only one bit set -> it's just the point at 'lb'
                    table_proj.push(points_affine[start + lb].to_proj());
                } else {
                    let mut new_val = table_proj[prev].clone();
                    new_val.add_or_dbl_assign(&points_affine[start + lb].to_proj());
                    table_proj.push(new_val);
                }
            }

            // Convert projective table entries back to affine and store
            // Use TG1Affine::into_affine? We have `to_affine` available on TG1::to_affine() or TG1Affine::into_affine(&proj)
            // We'll convert each TG1 proj to affine via TG1Affine::into_affine(&proj)
            // However, in earlier code they used TG1Affine::into_affine(&tmp_point) where tmp_point is TG1.
            // We'll use that.
            for p_proj in table_proj.into_iter() {
                table.push(TG1Affine::into_affine(&p_proj));
            }

            chunk_tables.push(table);
        }

        let table = StraussTable {
            chunk_tables,
            chunk_sizes,
            numpoints: n,
            batch_numpoints: 0,
            batch_points: Vec::new(),
            g1_marker: PhantomData,
            g1_fp_marker: PhantomData,
            fr_marker: PhantomData,
            g1_affine_add_marker: PhantomData,
        };

        Ok(Some(table))
    }

    /// Multiply using the precomputed chunk tables (sequential)
    pub fn multiply_sequential(&self, scalars: &[TFr]) -> TG1 {
        let n = scalars.len();
        if n == 0 || self.chunk_tables.is_empty() {
            return TG1::zero();
        }

        // Convert scalars to scalar limbs for bit access
        let scalar_values = scalars.iter().map(TFr::to_scalar).collect::<Vec<_>>();

        // For each chunk, we will compute the chunk result using the precomputed table,
        // then add it into the global accumulator.
        let mut accumulator = TG1::zero();

        let mut pt_idx = 0usize;
        for (chunk_idx, table) in self.chunk_tables.iter().enumerate() {
            let chunk_len = self.chunk_sizes[chunk_idx];
            if chunk_len == 0 {
                continue;
            }

            // Prepare mask bit extraction for this chunk.
            // We need to process all bits from MSB down to 0 for the scalars restricted to this chunk's indices.
            // For Strauss we compute table[mask] where mask is built from the current bit of each scalar in the chunk.
            // But we can do the original algorithm more simply:
            // - find max bit across scalars limited to these indices
            // - for bit from max..0:
            //    - dbl accumulator
            //    - build mask from chunk_len scalars at this bit
            //    - add table[mask]
            // However building mask from scratch for each bit is direct.

            // Compute max bit among involved scalars
            let mut max_bit = 0usize;
            for i in 0..chunk_len {
                let scalar_idx = pt_idx + i;
                if scalar_idx >= scalar_values.len() {
                    break;
                }
                let s = &scalar_values[scalar_idx];
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
            }

            // local loop: process bits MSB..0
            let mut chunk_acc = TG1::zero();
            for bit in (0..=max_bit).rev() {
                chunk_acc.dbl_assign();

                // build mask for this bit across chunk scalars
                let mut mask = 0usize;
                let limb_idx = bit / 64;
                let bit_in_limb = bit % 64;

                for i in 0..chunk_len {
                    let scalar_idx = pt_idx + i;
                    if scalar_idx >= scalar_values.len() {
                        break;
                    }

                    let s = &scalar_values[scalar_idx];
                    if limb_idx < s.data.len() {
                        if ((s.data[limb_idx] >> bit_in_limb) & 1) != 0 {
                            mask |= 1 << i;
                        }
                    }
                }

                if mask != 0 {
                    // table[mask] is affine; convert to proj and add
                    let tab_aff = &table[mask];
                    chunk_acc.add_or_dbl_assign(&tab_aff.to_proj());
                }
            }

            // Add the chunk result to the global accumulator
            accumulator.add_or_dbl_assign(&chunk_acc);

            pt_idx += chunk_len;
        }

        accumulator
    }

    pub fn multiply_batch(&self, scalars: &[Vec<TFr>]) -> Vec<TG1> {
        scalars.iter().map(|s| self.multiply_sequential(s)).collect()
    }

    #[cfg(feature = "parallel")]
    pub fn multiply_parallel(&self, scalars: &[TFr]) -> TG1 {
        // For now use the sequential method; can be parallelized later.
        self.multiply_sequential(scalars)
    }

    // Optional: persist precomputed chunk tables to file (requires `file-io` feature,
    // and the concrete TG1Affine type must implement serde Serialize/Deserialize)
    #[cfg(feature = "file-io")]
    pub fn save_to_file<S>(&self, path: S) -> Result<(), String>
    where
        S: AsRef<str>,
        TG1Affine: Serialize,
    {
        // Convert chunk tables into serializable shape
        let ser = SerializableChunkTable {
            chunk_sizes: self.chunk_sizes.clone(),
            chunk_tables: self
                .chunk_tables
                .iter()
                .map(|chunk| chunk.clone())
                .collect::<Vec<_>>(),
        };

        let bytes = bincode::serialize(&ser).map_err(|e| format!("serialize error: {}", e))?;
        std::fs::write(path.as_ref(), &bytes)
            .map_err(|e| format!("file write error: {}", e))?;
        Ok(())
    }

    // Optional: load from file (requires TG1Affine: DeserializeOwned + Clone)
    #[cfg(feature = "file-io")]
    pub fn load_from_file<S>(path: S) -> Result<Self, String>
    where
        S: AsRef<str>,
        TG1Affine: for<'de> Deserialize<'de> + Clone,
    {
        let bytes = std::fs::read(path.as_ref()).map_err(|e| format!("file read error: {}", e))?;
        let ser: SerializableChunkTable<TG1Affine> =
            bincode::deserialize(&bytes).map_err(|e| format!("deserialize error: {}", e))?;

        let table = StraussTable {
            chunk_tables: ser.chunk_tables,
            chunk_sizes: ser.chunk_sizes,
            numpoints: 0, // We don't know total numpoints here; user can set or extend if desired
            batch_numpoints: 0,
            batch_points: Vec::new(),
            g1_marker: PhantomData,
            g1_fp_marker: PhantomData,
            fr_marker: PhantomData,
            g1_affine_add_marker: PhantomData,
        };

        Ok(table)
    }
}
