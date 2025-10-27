use core::{
    num::Wrapping,
    sync::atomic::{AtomicUsize, Ordering},
};

use alloc::sync::Arc;
use std::sync::{mpsc::channel, Barrier};

use crate::{G1Affine, G1Fp, G1GetFp, Scalar256, G1};

use super::{
    cell::Cell,
    parallel_pippenger_utils::breakdown,
    pippenger_utils::{pippenger_window_size, P1XYZZ},
    thread_pool::{da_pool, ThreadPoolExt},
    tiling_pippenger_ops::{p1s_tile_pippenger_pub, tiling_pippenger},
};

struct Tile {
    x: usize,
    dx: usize,
    y: usize,
    dy: usize,
}

pub fn parallel_affine_conv<TG1: G1, TFp: G1Fp, TG1Affine: G1Affine<TG1, TFp> + Sized>(
    points: &[TG1],
) -> Vec<TG1Affine> {
    let npoints = points.len();
    let pool = da_pool();
    let ncpus = pool.max_count();
    if ncpus < 2 || npoints < 768 {
        return TG1Affine::into_affines(points);
    }

    let mut ret = Vec::<TG1Affine>::with_capacity(npoints);
    #[allow(clippy::uninit_vec)]
    unsafe {
        ret.set_len(npoints)
    };

    let mut nslices = npoints.div_ceil(512);
    nslices = core::cmp::min(nslices, ncpus);
    let wg = Arc::new((Barrier::new(2), AtomicUsize::new(nslices)));

    let (mut delta, mut rem) = (npoints / nslices + 1, Wrapping(npoints % nslices));
    let mut x = 0usize;
    while x < npoints {
        delta -= (rem == Wrapping(0)) as usize;
        rem -= Wrapping(1);

        let out = &mut ret[x..x + delta];
        let inp = &points[x..x + delta];

        x += delta;

        let wg = wg.clone();
        pool.joined_execute(move || {
            TG1Affine::into_affines_loc(out, inp);
            if wg.1.fetch_sub(1, Ordering::AcqRel) == 1 {
                wg.0.wait();
            }
        });
    }
    wg.0.wait();

    ret
}

pub fn tiling_parallel_pippenger<
    TG1: G1 + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
>(
    mut points: &[TG1Affine],
    scalars: &[Scalar256],
) -> TG1 {
    if scalars.len() < points.len() {
        points = &points[0..scalars.len()];
    }
    let npoints = points.len();

    let pool = da_pool();
    let ncpus = pool.max_count();

    if ncpus < 2 || npoints < 32 {
        return tiling_pippenger(points, scalars);
    }

    let (nx, ny, window) = breakdown(pippenger_window_size(npoints), ncpus);

    // |grid[]| holds "coordinates" and place for result
    let mut grid: Vec<(Tile, Cell<TG1>)> = Vec::with_capacity(nx * ny);
    #[allow(clippy::uninit_vec)]
    unsafe {
        grid.set_len(grid.capacity())
    };
    let dx = npoints / nx;
    let mut y = window * (ny - 1);
    let mut total = 0usize;

    while total < nx {
        grid[total].0.x = total * dx;
        grid[total].0.dx = dx;
        grid[total].0.y = y;
        grid[total].0.dy = 255 - y;
        total += 1;
    }
    grid[total - 1].0.dx = npoints - grid[total - 1].0.x;
    while y != 0 {
        y -= window;
        for i in 0..nx {
            grid[total].0.x = grid[i].0.x;
            grid[total].0.dx = grid[i].0.dx;
            grid[total].0.y = y;
            grid[total].0.dy = window;
            total += 1;
        }
    }
    let grid = &grid[..];

    let points = points;

    let mut row_sync: Vec<AtomicUsize> = Vec::with_capacity(ny);
    row_sync.resize_with(ny, Default::default);
    let row_sync = Arc::new(row_sync);
    let counter = Arc::new(AtomicUsize::new(0));
    let (tx, rx) = channel();
    let n_workers = core::cmp::min(ncpus, total);
    for _ in 0..n_workers {
        let tx = tx.clone();
        let counter = counter.clone();
        let row_sync = row_sync.clone();

        pool.joined_execute(move || {
            let mut buckets = vec![P1XYZZ::<TG1Fp>::default(); 1 << (window - 1)];
            loop {
                let work = counter.fetch_add(1, Ordering::Relaxed);
                if work >= total {
                    break;
                }

                let x = grid[work].0.x;
                let y = grid[work].0.y;
                let dx = grid[work].0.dx;

                p1s_tile_pippenger_pub(
                    grid[work].1.as_mut(),
                    &points[x..(x + dx)],
                    &scalars[x..],
                    &mut buckets,
                    y,
                    window,
                );
                if row_sync[y / window].fetch_add(1, Ordering::AcqRel) == nx - 1 {
                    tx.send(y).expect("disaster");
                }
            }
        });
    }

    let mut ret = <TG1>::default();
    let mut rows = vec![false; ny];
    let mut row = 0usize;
    for _ in 0..ny {
        let mut y = rx.recv().unwrap();
        rows[y / window] = true;
        while grid[row].0.y == y {
            while row < total && grid[row].0.y == y {
                ret.add_or_dbl_assign(grid[row].1.as_mut());
                row += 1;
            }
            if y == 0 {
                break;
            }
            for _ in 0..window {
                ret.dbl_assign();
            }
            y -= window;
            if !rows[y / window] {
                break;
            }
        }
    }
    ret
}
