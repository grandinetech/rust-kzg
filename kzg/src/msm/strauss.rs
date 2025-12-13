// Strauss fixed-window MSM implementation matching the style of wbits.rs / bgmw.rs

use core::marker::PhantomData;

use crate::{Fr, G1Affine, G1Fp, G1GetFp, G1Mul, G1ProjAddAffine, G1};

#[derive(Debug, Clone)]
pub struct StraussTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>
where
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TG1ProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
{
    numpoints: usize,
    window: usize,
    subtable_size: usize,

    /// Flat storage: for point i, its subtable lives at
    /// points[i * subtable_size .. (i+1) * subtable_size]
    points: Vec<TG1Affine>,

    batch_numpoints: usize,
    batch_points: Vec<Vec<TG1Affine>>,

    g1_marker: PhantomData<TG1>,
    g1_fp_marker: PhantomData<TG1Fp>,
    fr_marker: PhantomData<TFr>,
    g1_affine_add_marker: PhantomData<TG1ProjAddAffine>,
}

impl<
        TFr: Fr,
        TG1Fp: G1Fp,
        TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
        TG1Affine: G1Affine<TG1, TG1Fp>,
        TG1ProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
    > StraussTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>
{
    /// Read WINDOW_SIZE from environment at runtime (default 7)
    fn read_window_size() -> usize {
        std::env::var("WINDOW_SIZE")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|&v| v >= 1 && v <= 40)
            .unwrap_or(7usize)
    }

    /// Build the Strauss precomputation tables.
    /// Stores multiples 1..(2^w - 1) for each base point as affine.
    pub fn new(points: &[TG1], matrix: &[Vec<TG1>]) -> Result<Option<Self>, String> {
        let window = Self::read_window_size();
        if window >= 40 {
            return Err("WINDOW_SIZE too large".to_owned());
        }

        let subtable_size = (1usize << window).saturating_sub(1);

        // Flat table reserve
        let mut table: Vec<TG1Affine> = Vec::new();
        table
            .try_reserve_exact(points.len().saturating_mul(subtable_size))
            .map_err(|_| "Strauss precomputation table is too large".to_string())?;

        for p in points.iter() {
            // compute multiples: 1..(2^w - 1)
            let mut cur = p.clone();
            for _ in 0..subtable_size {
                table.push(TG1Affine::into_affine(&cur));
                cur = cur.add_or_dbl(p);
            }
        }

        if matrix.is_empty() {
            Ok(Some(Self {
                numpoints: points.len(),
                window,
                subtable_size,
                points: table,
                batch_numpoints: 0,
                batch_points: Vec::new(),
                g1_marker: PhantomData,
                g1_fp_marker: PhantomData,
                fr_marker: PhantomData,
                g1_affine_add_marker: PhantomData,
            }))
        } else {
            let batch_numpoints = matrix[0].len();
            let mut batch_points: Vec<Vec<TG1Affine>> = Vec::new();
            batch_points
                .try_reserve_exact(matrix.len())
                .map_err(|_| "Strauss precomputation table is too large".to_owned())?;

            for row in matrix {
                let mut temp: Vec<TG1Affine> = Vec::new();
                temp.try_reserve_exact(row.len().saturating_mul(subtable_size))
                    .map_err(|_| "Strauss precomputation table is too large".to_owned())?;

                for p in row {
                    let mut cur = p.clone();
                    for _ in 0..subtable_size {
                        temp.push(TG1Affine::into_affine(&cur));
                        cur = cur.add_or_dbl(p);
                    }
                }

                batch_points.push(temp);
            }

            Ok(Some(Self {
                numpoints: points.len(),
                window,
                subtable_size,
                points: table,
                batch_numpoints,
                batch_points,
                g1_marker: PhantomData,
                g1_fp_marker: PhantomData,
                fr_marker: PhantomData,
                g1_affine_add_marker: PhantomData,
            }))
        }
    }

    /// Extract a window (w bits) starting at window_idx*w (LSB windows)
    #[inline(always)]
    fn extract_window_bits(scalar_bytes: &[u8], window_idx: usize, w: usize) -> usize {
        let skip_bits = window_idx * w;
        let skip_bytes = skip_bits / 8;
        let mut v: u64 = 0;
        let max_take = core::cmp::min(8, scalar_bytes.len().saturating_sub(skip_bytes));
        for i in 0..max_take {
            v |= (scalar_bytes[skip_bytes + i] as u64) << (8 * i);
        }
        let shift = skip_bits - (skip_bytes * 8);
        v >>= shift;
        let mask = if w == 64 { u64::MAX } else { (1u64 << w) - 1 };
        (v & mask) as usize
    }

    fn multiply_sequential_raw(
        bases: &[TG1Affine],
        scalars: &[TFr],
        window: usize,
        subtable_size: usize,
    ) -> TG1 {
        let scalars_bytes = scalars.iter().map(TFr::to_scalar).collect::<Vec<_>>();
        let number_of_windows = 255usize.div_ceil(window);

        // gather windows points (affine) for each window
        let mut windows_of_points: Vec<Vec<TG1Affine>> =
            vec![Vec::with_capacity(scalars.len()); number_of_windows];

        for window_idx in 0..number_of_windows {
            for (scalar_idx, scalar_bytes) in scalars_bytes.iter().enumerate() {
                let bytes = scalar_bytes.as_u8();
                let digit = Self::extract_window_bits(bytes, window_idx, window);
                if digit == 0 {
                    continue;
                }
                let point_offset = scalar_idx.saturating_mul(subtable_size);
                let idx = point_offset + (digit - 1);
                // safe because table was sized accordingly in new()
                let pt = bases[idx];
                windows_of_points[window_idx].push(pt);
            }
        }

        // reduce each window into a single TG1 accumulator using affine -> proj add
        let mut accumulated_windows: Vec<TG1> = windows_of_points
            .into_iter()
            .map(|pts| {
                let mut acc = TG1::identity();
                for p in pts {
                    TG1ProjAddAffine::add_or_double_assign_affine(&mut acc, &p);
                }
                acc
            })
            .collect();

        // fold from high window down
        let mut result: TG1 = accumulated_windows.last().unwrap().clone();
        for acc in accumulated_windows.into_iter().rev().skip(1) {
            for _ in 0..window {
                result = result.dbl();
            }
            result.add_or_dbl_assign(&acc);
        }

        result
    }

    pub fn multiply_sequential(&self, scalars: &[TFr]) -> TG1 {
        Self::multiply_sequential_raw(&self.points, scalars, self.window, self.subtable_size)
    }

    pub fn multiply_batch(&self, scalars: &[Vec<TFr>]) -> Vec<TG1> {
        assert!(scalars.len() == self.batch_points.len());

        #[cfg(not(feature = "parallel"))]
        {
            self.batch_points
                .iter()
                .zip(scalars)
                .map(|(points, scalars)| {
                    Self::multiply_sequential_raw(points, scalars, self.window, self.subtable_size)
                })
                .collect::<Vec<_>>()
        }

        #[cfg(feature = "parallel")]
        {
            use super::{
                cell::Cell,
                thread_pool::{da_pool, ThreadPoolExt},
            };
            use core::sync::atomic::{AtomicUsize, Ordering};
            use std::sync::Arc;

            let pool = da_pool();
            let ncpus = pool.max_count();
            let counter = Arc::new(AtomicUsize::new(0));
            let mut results: Vec<Cell<TG1>> = Vec::with_capacity(scalars.len());
            #[allow(clippy::uninit_vec)]
            unsafe {
                results.set_len(results.capacity())
            };
            let results = &results[..];

            for _ in 0..ncpus {
                let counter = counter.clone();
                pool.joined_execute(move || loop {
                    let work = counter.fetch_add(1, Ordering::Relaxed);
                    if work >= scalars.len() {
                        break;
                    }
                    let result = Self::multiply_sequential_raw(
                        &self.batch_points[work],
                        &scalars[work],
                        self.window,
                        self.subtable_size,
                    );
                    unsafe { *results[work].as_ptr().as_mut().unwrap() = result };
                });
            }

            pool.join();

            results.iter().map(|it| it.as_mut().clone()).collect()
        }
    }

    #[cfg(feature = "parallel")]
    pub fn multiply_parallel(&self, scalars: &[TFr]) -> TG1 {
        // For now reuse sequential implementation; can be replaced with tiling approach.
        self.multiply_sequential(scalars)
    }
}
