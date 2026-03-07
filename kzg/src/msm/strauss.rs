use crate::msm::pippenger_utils::get_wval_limb;
use crate::{Fr, G1Affine, G1Fp, G1GetFp, G1Mul, G1ProjAddAffine, G1};
use alloc::vec::Vec;
use core::marker::PhantomData;

#[cfg(feature = "diskcache")]
use crate::msm::diskcache::DiskCache;

// Strauss chunk size: process this many points at a time.
// Table size = 2^CHUNK_SIZE. For CHUNK_SIZE=7: table has 128 entries.

fn get_window_size() -> usize {
    option_env!("WINDOW_SIZE")
        .map(|v| {
            v.parse()
                .expect("WINDOW_SIZE environment variable must be valid number")
        })
        .unwrap_or(8)
}

#[derive(Debug, Clone)]
pub struct StraussTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>
where
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TG1ProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
{
    chunk_tables: Vec<Vec<TG1Affine>>,
    numpoints: usize,

    batch_numpoints: usize,
    batch_chunk_tables: Vec<Vec<Vec<TG1Affine>>>, // precomputed tables per row

    g1_marker: PhantomData<TG1>,
    g1_fp_marker: PhantomData<TG1Fp>,
    fr_marker: PhantomData<TFr>,
    g1_affine_marker: PhantomData<TG1Affine>,
    g1_affine_add_marker: PhantomData<TG1ProjAddAffine>,
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
    fn try_read_cache(points: &[TG1], matrix: &[Vec<TG1>]) -> Result<Self, Option<[u8; 32]>> {
        #[cfg(feature = "diskcache")]
        {
            DiskCache::<TG1, TG1Fp, TG1Affine>::load("strauss", get_window_size(), points, matrix)
                .map_err(|(err, contenthash)| {
                    println!("Failed to load cache: {err}");
                    contenthash
                })
                .map(|cache| {
                    // Reconstruct chunk_tables from cache
                    let chunk_size = get_window_size();
                    let n = cache.numpoints;
                    let num_chunks = n.div_ceil(chunk_size);

                    let mut chunk_tables = Vec::new();
                    let mut offset = 0;

                    for chunk_idx in 0..num_chunks {
                        let start = chunk_idx * chunk_size;
                        let end = core::cmp::min(start + chunk_size, n);
                        let chunk_len = end - start;
                        let table_size = (1usize << chunk_len) - 1;

                        // Store directly as affine
                        let chunk: Vec<TG1Affine> =
                            cache.table[offset..offset + table_size].to_vec();
                        chunk_tables.push(chunk);
                        offset += table_size;
                    }

                    // Rebuild batch_chunk_tables from cache
                    // DiskCache stores batch_table as Vec<Vec<TG1Affine>> - each row is already flattened
                    let batch_chunk_tables: Vec<Vec<Vec<TG1Affine>>> = cache
                        .batch_table
                        .iter()
                        .map(|flat_row| {
                            Self::unflatten_row_chunk_tables(
                                flat_row,
                                cache.batch_numpoints,
                                chunk_size,
                            )
                        })
                        .collect();

                    Self {
                        chunk_tables,
                        numpoints: cache.numpoints,
                        batch_numpoints: cache.batch_numpoints,
                        batch_chunk_tables,
                        g1_marker: PhantomData,
                        g1_fp_marker: PhantomData,
                        fr_marker: PhantomData,
                        g1_affine_marker: PhantomData,
                        g1_affine_add_marker: PhantomData,
                    }
                })
        }

        #[cfg(not(feature = "diskcache"))]
        Err(None)
    }

    fn try_write_cache(
        points: &[TG1],
        matrix: &[Vec<TG1>],
        chunk_tables: &[Vec<TG1Affine>],
        numpoints: usize,
        batch_chunk_tables: &[Vec<Vec<TG1Affine>>],
        batch_numpoints: usize,
        contenthash: Option<[u8; 32]>,
    ) -> Result<(), String> {
        #[cfg(feature = "diskcache")]
        {
            // Flatten chunk_tables
            let table_affine: Vec<TG1Affine> = chunk_tables
                .iter()
                .flat_map(|chunk| chunk.iter())
                .cloned()
                .collect();

            // Flatten each row's chunk_tables for DiskCache's 2D structure
            let batch_table_affine: Vec<Vec<TG1Affine>> = batch_chunk_tables
                .iter()
                .map(|row_chunks| {
                    row_chunks
                        .iter()
                        .flat_map(|chunk| chunk.iter())
                        .cloned()
                        .collect()
                })
                .collect();

            DiskCache::<TG1, TG1Fp, TG1Affine>::save(
                "strauss",
                get_window_size(),
                points,
                matrix,
                &table_affine,
                numpoints,
                &batch_table_affine,
                batch_numpoints,
                contenthash,
            )
            .inspect_err(|err| println!("Failed to save cache: {err}"))
        }

        #[cfg(not(feature = "diskcache"))]
        Ok(())
    }

    fn unflatten_row_chunk_tables(
        flat_row: &[TG1Affine],
        batch_numpoints: usize,
        chunk_size: usize,
    ) -> Vec<Vec<TG1Affine>> {
        if flat_row.is_empty() {
            return Vec::new();
        }

        let num_chunks = batch_numpoints.div_ceil(chunk_size);
        let mut row_chunk_tables = Vec::new();
        let mut offset = 0;

        for chunk_idx in 0..num_chunks {
            let start = chunk_idx * chunk_size;
            let end = core::cmp::min(start + chunk_size, batch_numpoints);
            let chunk_len = end - start;
            let table_size = (1usize << chunk_len) - 1;

            let chunk: Vec<TG1Affine> = flat_row[offset..offset + table_size].to_vec();
            row_chunk_tables.push(chunk);
            offset += table_size;
        }

        row_chunk_tables
    }

    /// Build a StraussTable that precomputes chunk tables for all chunks.
    /// This mirrors the style of other algorithms that precompute and store tables.
    pub fn new(points: &[TG1], matrix: &[Vec<TG1>]) -> Result<Option<Self>, String> {
        let contenthash = match Self::try_read_cache(points, matrix) {
            Ok(v) => return Ok(Some(v)),
            Err(e) => e,
        };

        let strauss_chunk_size: usize = get_window_size();

        // If matrix is empty, build single-point-set precomputation
        if matrix.is_empty() {
            let n = points.len();
            if n == 0 {
                let table = StraussTable {
                    chunk_tables: Vec::new(),
                    numpoints: 0,
                    batch_numpoints: 0,
                    batch_chunk_tables: Vec::new(),
                    g1_marker: PhantomData,
                    g1_fp_marker: PhantomData,
                    fr_marker: PhantomData,
                    g1_affine_marker: PhantomData,
                    g1_affine_add_marker: PhantomData,
                };
                return Ok(Some(table));
            }

            // Build chunk tables as affine
            let chunk_tables = Self::build_chunk_tables(points, strauss_chunk_size);

            Self::try_write_cache(points, matrix, &chunk_tables, n, &[], 0, contenthash)?;

            let table = StraussTable {
                chunk_tables,
                numpoints: n,
                batch_numpoints: 0,
                batch_chunk_tables: Vec::new(),
                g1_marker: PhantomData,
                g1_fp_marker: PhantomData,
                fr_marker: PhantomData,
                g1_affine_marker: PhantomData,
                g1_affine_add_marker: PhantomData,
            };
            return Ok(Some(table));
        }

        let batch_numpoints = matrix[0].len();

        // Build chunk tables for each row
        let batch_chunk_tables: Vec<Vec<Vec<TG1Affine>>> = matrix
            .iter()
            .map(|point_row| Self::build_chunk_tables(point_row, strauss_chunk_size))
            .collect();

        // Build main chunk_tables if needed
        let n = points.len();
        let chunk_tables = if n > 0 {
            Self::build_chunk_tables(points, strauss_chunk_size)
        } else {
            Vec::new()
        };

        Self::try_write_cache(
            points,
            matrix,
            &chunk_tables,
            n,
            &batch_chunk_tables,
            batch_numpoints,
            contenthash,
        )?;

        let table = StraussTable {
            chunk_tables,
            numpoints: n,
            batch_numpoints,
            batch_chunk_tables,
            g1_marker: PhantomData,
            g1_fp_marker: PhantomData,
            fr_marker: PhantomData,
            g1_affine_marker: PhantomData,
            g1_affine_add_marker: PhantomData,
        };
        Ok(Some(table))
    }

    /// Build chunk tables - returns AFFINE for storage efficiency
    fn build_chunk_tables(points: &[TG1], chunk_size: usize) -> Vec<Vec<TG1Affine>> {
        let n = points.len();
        let mut chunk_tables: Vec<Vec<TG1Affine>> = Vec::new();

        let num_chunks = n.div_ceil(chunk_size);

        for chunk_idx in 0..num_chunks {
            let start = chunk_idx * chunk_size;
            let end = core::cmp::min(start + chunk_size, n);
            let chunk_len = end - start;

            // size of table for this chunk: 2^chunk_len entries, but we skip index 0 (identity)
            let table_size = (1usize << chunk_len) - 1;

            // Build incremental table in projective space using the lowest-bit trick.
            // faster additions in projective
            let mut table_proj: Vec<TG1> = Vec::with_capacity(table_size);

            for mask in 1..=table_size {
                let lb = mask.trailing_zeros() as usize;
                let prev = mask ^ (1 << lb);
                if prev == 0 {
                    table_proj.push(points[start + lb].clone());
                } else {
                    let mut new_val = table_proj[prev - 1].clone();
                    new_val.add_or_dbl_assign(&points[start + lb]);
                    table_proj.push(new_val);
                }
            }

            // Convert to affine once for storage
            let table_affine: Vec<TG1Affine> = table_proj
                .iter()
                .map(|proj| TG1Affine::into_affine(proj))
                .collect();

            chunk_tables.push(table_affine);
        }

        chunk_tables
    }

    /// Multiply using the precomputed chunk tables (sequential)
    pub fn multiply_sequential(&self, scalars: &[TFr]) -> TG1 {
        Self::multiply_with_tables(scalars, &self.chunk_tables)
    }

    /// Core multiplication logic using provided tables
    fn multiply_with_tables(scalars: &[TFr], chunk_tables: &[Vec<TG1Affine>]) -> TG1 {
        let n = scalars.len();
        if n == 0 || chunk_tables.is_empty() {
            return TG1::zero();
        }

        // Convert scalars to scalar limbs for bit access
        let scalar_values = scalars.iter().map(TFr::to_scalar).collect::<Vec<_>>();

        // Single accumulator processing all chunks together
        let mut accumulator = TG1::zero();

        // Process all 255 bits (BLS12-381 scalar bit length)
        for bit in (0..255).rev() {
            // Double accumulator unconditionally
            accumulator.dbl_assign();

            // Process each chunk at this bit position
            let mut pt_idx = 0usize;
            for table in chunk_tables.iter() {
                let table_size = table.len();
                // Derive chunk_len from table size: table_size = 2^chunk_len - 1
                // So 2^chunk_len = table_size + 1, thus log2(table_size + 1)
                let chunk_len =
                    (usize::BITS - 1) as usize - (table_size + 1).leading_zeros() as usize;

                // Only process this chunk if we have scalars for it
                // This handles the case where tables were built for more points than we're using
                if pt_idx >= scalar_values.len() {
                    break;
                }

                // Build table_index for this bit across chunk scalars
                let mut table_index = 0usize;
                let actual_chunk_len = core::cmp::min(chunk_len, scalar_values.len() - pt_idx);

                for i in 0..actual_chunk_len {
                    let scalar_idx = pt_idx + i;

                    let s = &scalar_values[scalar_idx];
                    // Extract single bit at position 'bit' from scalar
                    if (get_wval_limb(s, bit, 1) & 1) != 0 {
                        table_index |= 1 << i;
                    }
                }

                if table_index != 0 {
                    // Mixed addition - Projective + Affine (should be faster than Proj + Proj)
                    let affine_pt = &table[table_index - 1];
                    TG1ProjAddAffine::add_or_double_assign_affine(&mut accumulator, affine_pt);
                }

                pt_idx += chunk_len;
            }
        }

        accumulator
    }

    pub fn multiply_batch(&self, scalars: &[Vec<TFr>]) -> Vec<TG1> {
        // Use precomputed batch_chunk_tables
        assert!(
            scalars.len() == self.batch_chunk_tables.len(),
            "Scalars length {} != batch_chunk_tables length {}",
            scalars.len(),
            self.batch_chunk_tables.len()
        );

        scalars
            .iter()
            .zip(self.batch_chunk_tables.iter())
            .map(|(scalar_row, chunk_tables)| Self::multiply_with_tables(scalar_row, chunk_tables))
            .collect()
    }

    pub fn multiply_parallel(&self, scalars: &[TFr]) -> TG1 {
        self.multiply_sequential(scalars)
    }
}
