use core::marker::PhantomData;

use crate::{Fr, G1Affine, G1Fp, G1GetFp, G1Mul, G1ProjAddAffine, Scalar256, G1};

use super::pippenger_utils::{
    booth_decode, booth_encode, get_wval_limb, is_zero, num_bits, p1_dadd, p1_to_jacobian,
    type_is_zero, P1XYZZ,
};

#[derive(Debug, Clone)]
pub struct BgmwTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>
where
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TG1ProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
{
    window: BgmwWindow,
    points: Vec<TG1Affine>,
    numpoints: usize,
    h: usize,

    batch_window: BgmwWindow,
    batch_points: Vec<Vec<TG1Affine>>,
    batch_numpoints: usize,
    batch_h: usize,

    g1_marker: PhantomData<TG1>,
    g1_fp_marker: PhantomData<TG1Fp>,
    fr_marker: PhantomData<TFr>,
    g1_affine_add_marker: PhantomData<TG1ProjAddAffine>,
}

const NBITS: usize = 255;

#[cfg(feature = "parallel")]
#[derive(Debug, Clone, Copy)]
enum BgmwWindow {
    Sync(usize),
    Parallel((usize, usize, usize)),
}

#[cfg(not(feature = "parallel"))]
type BgmwWindow = usize;

#[inline]
const fn get_table_dimensions(window: BgmwWindow) -> (usize, usize) {
    let window_width;

    #[cfg(not(feature = "parallel"))]
    {
        window_width = window;
    }

    #[cfg(feature = "parallel")]
    {
        window_width = match window {
            BgmwWindow::Sync(wnd) => wnd,
            BgmwWindow::Parallel((_, ny, wnd)) => return (wnd, ny),
        }
    }

    let h = NBITS.div_ceil(window_width) + is_zero((NBITS % window_width) as u64) as usize;

    (window_width, h)
}

#[inline]
const fn get_sequential_window_size(window: BgmwWindow) -> usize {
    #[cfg(not(feature = "parallel"))]
    {
        window
    }

    #[cfg(feature = "parallel")]
    {
        match window {
            BgmwWindow::Sync(wnd) => wnd,
            BgmwWindow::Parallel(_) => {
                panic!("Cannot use parallel BGMW table in sequential version")
            }
        }
    }
}

/// Function, which approximates minimum of this function:
/// y = ceil(255/w) * (npoints) + 2^w - 2
/// This function is number of additions and doublings required to compute msm using Pippenger algorithm, with BGMW
/// precomputation table.
/// Parts of this function:
///   ceil(255/w) - how many parts will be in decomposed scalar. Scalar width is 255 bits, so converting it into q-ary
///                 representation, will produce 255/w parts. q-ary representation, where q = 2^w, for scalar a is:
///                 a = a_1 + a_2 * q + ... + a_n * q^(ceil(255/w)).
///   npoints     - each scalar must be assigned to a bucket (bucket accumulation). Assigning point to bucket means
///                 adding it to existing point in bucket - hence, the addition.
///   2^w - 2     - computing total bucket sum (bucket aggregation). Total number of buckets (scratch size) is 2^(w-1).
///                 Adding each point to total bucket sum requires 2 point addition operations, so 2 * 2^(w-1) = 2^w.
#[allow(unused)]
const fn bgmw_window_size(npoints: usize) -> usize {
    let wbits = num_bits(npoints);

    match (wbits) {
        1 => 4,
        2..=3 => 5,
        4 => 6,
        5 => 7,
        6..=7 => 8,
        8 => 9,
        9..=10 => 10,
        11 => 11,
        12 => 12,
        13..=14 => 13,
        15..=16 => 15,
        17 => 16,
        18..=19 => 17,
        20 => 19,
        21..=22 => 20,
        23..=24 => 22,
        25..=26 => 24,
        27..=29 => 26,
        30..=32 => 29,
        33..=37 => 32,
        _ => 37,
    }
}

#[cfg(feature = "parallel")]
const fn bgmw_parallel_window_size(npoints: usize, ncpus: usize) -> (usize, usize, usize) {
    let mut min_ops = usize::MAX;
    let mut opt = 0;

    let mut win = 2;
    while win <= 40 {
        let ops = (1 << win) + (255usize.div_ceil(win).div_ceil(ncpus) * npoints) - 2;
        if min_ops >= ops {
            min_ops = ops;
            opt = win;
        }
        win += 1;
    }

    let mut mult = 1;

    let mut opt_x = 1;

    while mult <= 8 {
        let nx = ncpus * mult;
        let wnd = bgmw_window_size(npoints / nx);

        let ops = mult * 255usize.div_ceil(wnd) * npoints.div_ceil(nx) + (1 << wnd) - 2;

        if min_ops > ops {
            min_ops = ops;
            opt = wnd;
            opt_x = nx;
        }

        mult += 1;
    }

    (
        opt_x,
        255usize.div_ceil(opt) + is_zero((NBITS % opt) as u64) as usize,
        opt,
    )
}

impl<
        TFr: Fr,
        TG1Fp: G1Fp,
        TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
        TG1Affine: G1Affine<TG1, TG1Fp>,
        TG1ProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
    > BgmwTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>
{
    pub fn new(points: &[TG1], matrix: &[Vec<TG1>]) -> Result<Option<Self>, String> {
        let window = Self::window(points.len());

        let (window_width, h) = get_table_dimensions(window);

        let mut table: Vec<TG1Affine> = Vec::new();
        let q = TFr::from_u64(1u64 << window_width);

        table
            .try_reserve_exact(points.len() * h)
            .map_err(|_| "BGMW precomputation table is too large".to_string())?;

        unsafe { table.set_len(points.len() * h) };

        for i in 0..points.len() {
            let mut tmp_point = points[i].clone();
            for j in 0..h {
                let idx = j * points.len() + i;
                table[idx] = TG1Affine::into_affine(&tmp_point);
                tmp_point = tmp_point.mul(&q);
            }
        }

        if matrix.is_empty() {
            Ok(Some(Self {
                numpoints: points.len(),
                points: table,
                window,
                h,

                batch_window: {
                    #[cfg(feature = "parallel")]
                    let w = BgmwWindow::Sync(0);

                    #[cfg(not(feature = "parallel"))]
                    let w = 0;

                    w
                },
                batch_numpoints: 0,
                batch_points: Vec::new(),
                batch_h: 0,

                fr_marker: PhantomData,
                g1_fp_marker: PhantomData,
                g1_marker: PhantomData,
                g1_affine_add_marker: PhantomData,
            }))
        } else {
            let batch_numpoints = matrix[0].len();
            let batch_window = Self::sequential_window(batch_numpoints);
            let (batch_window_width, batch_h) = get_table_dimensions(batch_window);
            let batch_q = TFr::from_u64(1u64 << batch_window_width);

            let mut batch_points = Vec::new();
            batch_points
                .try_reserve_exact(matrix.len())
                .map_err(|_| "BGMW precomputation table is too large".to_owned())?;

            for row in matrix {
                let mut temp_table = Vec::new();
                temp_table
                    .try_reserve_exact(row.len() * batch_h)
                    .map_err(|_| "BGMW precomputation table is too large".to_owned())?;

                unsafe {
                    temp_table.set_len(temp_table.capacity());
                }

                for i in 0..row.len() {
                    let mut tmp_point = row[i].clone();
                    for j in 0..batch_h {
                        let idx = j * row.len() + i;
                        temp_table[idx] = TG1Affine::into_affine(&tmp_point);
                        tmp_point = tmp_point.mul(&batch_q);
                    }
                }

                batch_points.push(temp_table);
            }

            Ok(Some(Self {
                numpoints: points.len(),
                points: table,
                window,
                h,

                batch_window,
                batch_numpoints,
                batch_points,
                batch_h,

                fr_marker: PhantomData,
                g1_fp_marker: PhantomData,
                g1_marker: PhantomData,
                g1_affine_add_marker: PhantomData,
            }))
        }
    }

    pub fn multiply_batch(&self, scalars: &[Vec<TFr>]) -> Vec<TG1> {
        assert!(scalars.len() == self.batch_points.len());

        #[cfg(not(feature = "parallel"))]
        {
            let window = get_sequential_window_size(self.batch_window);
            let mut buckets = vec![P1XYZZ::<TG1Fp>::default(); 1 << (window - 1)];

            self.batch_points
                .iter()
                .zip(scalars)
                .map(|(points, scalars)| {
                    Self::multiply_sequential_raw(
                        points,
                        scalars,
                        &mut buckets,
                        window,
                        self.batch_numpoints,
                        self.batch_h,
                    )
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

            let window = get_sequential_window_size(self.batch_window);

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
                pool.joined_execute(move || {
                    let mut buckets = vec![P1XYZZ::<TG1Fp>::default(); 1 << (window - 1)];

                    loop {
                        let work = counter.fetch_add(1, Ordering::Relaxed);

                        if work >= scalars.len() {
                            break;
                        }

                        let result = Self::multiply_sequential_raw(
                            &self.batch_points[work],
                            &scalars[work],
                            &mut buckets,
                            window,
                            self.batch_numpoints,
                            self.batch_h,
                        );
                        unsafe { *results[work].as_ptr().as_mut().unwrap() = result };
                    }
                });
            }

            pool.join();

            results.iter().map(|it| it.as_mut().clone()).collect()
        }
    }

    fn multiply_sequential_raw(
        points: &[TG1Affine],
        scalars: &[TFr],
        buckets: &mut [P1XYZZ<TG1Fp>],
        window: usize,
        numpoints: usize,
        h: usize,
    ) -> TG1 {
        let scalars = scalars.iter().map(TFr::to_scalar).collect::<Vec<_>>();
        let scalars = &scalars[..];

        let mut wbits: usize = 255 % window;
        let mut cbits: usize = wbits + 1;
        let mut bit0: usize = 255;

        let mut q_idx = h;

        loop {
            bit0 -= wbits;
            q_idx -= 1;
            if bit0 == 0 {
                break;
            }

            p1_tile_bgmw(
                &points[q_idx * numpoints..(q_idx + 1) * numpoints],
                scalars,
                buckets,
                bit0,
                wbits,
                cbits,
            );

            cbits = window;
            wbits = window;
        }
        p1_tile_bgmw(&points[0..numpoints], scalars, buckets, 0, wbits, cbits);

        let mut ret = TG1::default();
        integrate_buckets(&mut ret, buckets, wbits - 1);

        ret
    }

    pub fn multiply_sequential(&self, scalars: &[TFr]) -> TG1 {
        let window = get_sequential_window_size(self.window);
        let mut buckets = vec![P1XYZZ::<TG1Fp>::default(); 1 << (window - 1)];

        Self::multiply_sequential_raw(
            &self.points,
            scalars,
            &mut buckets,
            window,
            self.numpoints,
            self.h,
        )
    }

    #[cfg(feature = "parallel")]
    pub fn multiply_parallel(&self, scalars: &[TFr]) -> TG1 {
        use super::{
            cell::Cell,
            thread_pool::{da_pool, ThreadPoolExt},
            tiling_pippenger_ops::tiling_pippenger,
        };
        use core::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::{mpsc, Arc};

        let npoints = scalars.len();
        let pool = da_pool();
        let ncpus = pool.max_count();

        if ncpus > npoints || npoints < 32 {
            let scalars = scalars.iter().map(TFr::to_scalar).collect::<Vec<_>>();
            return tiling_pippenger(&self.points[0..npoints], &scalars);
        }

        struct Tile {
            x: usize,
            dx: usize,
            y: usize,
            dy: usize,
        }

        let (nx, ny, window) = match self.window {
            BgmwWindow::Sync(_) => return self.multiply_sequential(scalars),
            BgmwWindow::Parallel(values) => values,
        };

        let scalars = scalars.iter().map(TFr::to_scalar).collect::<Vec<_>>();
        let scalars = &scalars[..];

        // |grid[]| holds "coordinates"
        let mut grid: Vec<Tile> = Vec::with_capacity(nx * ny);
        #[allow(clippy::uninit_vec)]
        unsafe {
            grid.set_len(grid.capacity())
        };
        let dx = npoints / nx;
        let mut y = window * (ny - 1);
        let mut total = 0usize;

        while total < nx {
            grid[total].x = total * dx;
            grid[total].dx = dx;
            grid[total].y = y;
            grid[total].dy = NBITS - y;
            total += 1;
        }
        grid[total - 1].dx = npoints - grid[total - 1].x;
        while y != 0 {
            y -= window;
            for i in 0..nx {
                grid[total].x = grid[i].x;
                grid[total].dx = grid[i].dx;
                grid[total].y = y;
                grid[total].dy = window;
                total += 1;
            }
        }
        let grid = &grid[..];

        let mut row_sync: Vec<AtomicUsize> = Vec::with_capacity(ny);
        row_sync.resize_with(ny, Default::default);
        let counter = Arc::new(AtomicUsize::new(0));
        let (tx, rx) = mpsc::channel();
        let n_workers = core::cmp::min(ncpus, total);

        let mut results: Vec<Cell<TG1>> = Vec::with_capacity(n_workers);
        #[allow(clippy::uninit_vec)]
        unsafe {
            results.set_len(results.capacity());
        };

        let results = &results[..];

        #[allow(clippy::needless_range_loop)]
        for worker_index in 0..n_workers {
            let tx = tx.clone();
            let counter = counter.clone();

            pool.joined_execute(move || {
                let mut buckets = vec![P1XYZZ::<TG1Fp>::default(); 1 << (window - 1)];
                loop {
                    let work = counter.fetch_add(1, Ordering::Relaxed);
                    if work >= total {
                        integrate_buckets(
                            unsafe { results[worker_index].as_ptr().as_mut() }.unwrap(),
                            &mut buckets,
                            window - 1,
                        );
                        tx.send(worker_index).expect("disaster");

                        break;
                    }

                    let x = grid[work].x;
                    let y = grid[work].y;
                    let dx = grid[work].dx;

                    let row_start = (y / window) * self.numpoints + x;
                    let points = &self.points[row_start..(row_start + dx)];

                    let (wbits, cbits) = if y + window > NBITS {
                        let wbits = NBITS - y;
                        (wbits, wbits + 1)
                    } else {
                        (window, window)
                    };

                    p1_tile_bgmw(points, &scalars[x..(x + dx)], &mut buckets, y, wbits, cbits);
                }
            });
        }

        let mut ret = TG1::zero();
        for _ in 0..n_workers {
            let idx = rx.recv().unwrap();

            ret.add_or_dbl_assign(results[idx].as_mut());
        }
        ret
    }

    fn window(npoints: usize) -> BgmwWindow {
        #[cfg(feature = "parallel")]
        {
            use super::thread_pool::da_pool;

            let pool = da_pool();
            let ncpus = pool.max_count();

            if npoints >= 32 && ncpus >= 2 {
                BgmwWindow::Parallel(bgmw_parallel_window_size(npoints, ncpus))
            } else {
                BgmwWindow::Sync(bgmw_window_size(npoints))
            }
        }

        #[cfg(not(feature = "parallel"))]
        {
            bgmw_window_size(npoints)
        }
    }

    fn sequential_window(npoints: usize) -> BgmwWindow {
        #[cfg(feature = "parallel")]
        {
            BgmwWindow::Sync(bgmw_window_size(npoints))
        }

        #[cfg(not(feature = "parallel"))]
        {
            bgmw_window_size(npoints)
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn p1_tile_bgmw<TG1: G1 + G1GetFp<TFp>, TFp: G1Fp, TG1Affine: G1Affine<TG1, TFp>>(
    points: &[TG1Affine],
    scalars: &[Scalar256],
    buckets: &mut [P1XYZZ<TFp>],
    bit0: usize,
    wbits: usize,
    cbits: usize,
) {
    if scalars.is_empty() {
        return;
    }

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

    if scalars.len() == 1 {
        booth_decode(buckets, wval, cbits, point);
        return;
    }

    // Get second scalar
    let scalar = &scalars[1];

    // Calculate second window value (encoded bucket index)
    let wnxt = (get_wval_limb(scalar, bit0, wbits) << z) & wmask;
    let mut wnxt = booth_encode(wnxt, cbits);

    // Move first point to corresponding bucket
    booth_decode(buckets, wval, cbits, point);

    // Last point will be calculated separately, so decrementing point count
    let npoints = scalars.len() - 1;

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
}

/// Calculate bucket sum
///
/// This function multiplies the point in each bucket by it's index. Then, it will sum all multiplication results and write
/// resulting point to the `out`.
///
/// ## Arguments
///
/// * out     - output where bucket sum must be written
/// * buckets - pointer to the beginning of the array of buckets
/// * wbits   - window size, aka exponent of q (q^window)
///
fn integrate_buckets<TG1: G1 + G1GetFp<TFp>, TFp: G1Fp>(
    out: &mut TG1,
    buckets: &mut [P1XYZZ<TFp>],
    wbits: usize,
) {
    let mut n = (1usize << wbits) - 1;
    let mut ret = buckets[n];
    let mut acc = buckets[n];
    buckets[n] = P1XYZZ::<TFp>::default();

    loop {
        if n == 0 {
            break;
        }
        n -= 1;

        if type_is_zero(&buckets[n]) == 0 {
            p1_dadd(&mut acc, &buckets[n]);
        }
        buckets[n] = P1XYZZ::<TFp>::default();
        p1_dadd(&mut ret, &acc);
    }

    p1_to_jacobian(out, &ret);
}
