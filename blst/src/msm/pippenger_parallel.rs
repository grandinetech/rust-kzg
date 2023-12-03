// Copyright Supranational LLC
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0
extern crate alloc;
extern crate blst;
extern crate core;
extern crate threadpool;
use alloc::{boxed::Box, vec, vec::Vec};
use blst::{blst_p1, blst_p1_add_or_double, blst_p1_affine, blst_p1_double, blst_p1s_to_affine};
use core::num::Wrapping;
use core::ops::{Index, IndexMut};
use core::ptr;
use core::slice::SliceIndex;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc::channel, Arc, Barrier};

struct Tile {
    x: usize,
    dx: usize,
    y: usize,
    dy: usize,
}

trait ThreadPoolExt {
    fn joined_execute<'any, F>(&self, job: F)
    where
        F: FnOnce() + Send + 'any;
}

use core::mem::transmute;
use std::sync::{Mutex, Once};
use threadpool::ThreadPool;

use crate::types::g1::FsG1;

use super::pippenger::{pippenger_tile_pub, P1XYZZ};

pub fn da_pool() -> ThreadPool {
    static INIT: Once = Once::new();
    static mut POOL: *const Mutex<ThreadPool> = 0 as *const Mutex<ThreadPool>;

    INIT.call_once(|| {
        let pool = Mutex::new(ThreadPool::default());
        unsafe { POOL = transmute(Box::new(pool)) };
    });
    unsafe { (*POOL).lock().unwrap().clone() }
}

type Thunk<'any> = Box<dyn FnOnce() + Send + 'any>;

impl ThreadPoolExt for ThreadPool {
    fn joined_execute<'scope, F>(&self, job: F)
    where
        F: FnOnce() + Send + 'scope,
    {
        // Bypass 'lifetime limitations by brute force. It works,
        // because we explicitly join the threads...
        self.execute(unsafe { transmute::<Thunk<'scope>, Thunk<'static>>(Box::new(job)) })
    }
}

// Minimalist core::cell::Cell stand-in, but with Sync marker, which
// makes it possible to pass it to multiple threads. It works, because
// *here* each Cell is written only once and by just one thread.
#[repr(transparent)]
struct Cell<T: ?Sized> {
    value: T,
}
unsafe impl<T: ?Sized + Sync> Sync for Cell<T> {}
impl<T> Cell<T> {
    pub fn as_ptr(&self) -> *mut T {
        &self.value as *const T as *mut T
    }
}

//MULT IMPL
pub struct P1Affines {
    points: Vec<blst_p1_affine>,
}

impl<I: SliceIndex<[blst_p1_affine]>> Index<I> for P1Affines {
    type Output = I::Output;

    #[inline]
    fn index(&self, i: I) -> &Self::Output {
        &self.points[i]
    }
}
impl<I: SliceIndex<[blst_p1_affine]>> IndexMut<I> for P1Affines {
    #[inline]
    fn index_mut(&mut self, i: I) -> &mut Self::Output {
        &mut self.points[i]
    }
}

pub fn points_to_affine(points: &[FsG1]) -> Vec<blst_p1_affine> {
    let npoints = points.len();
    let mut ret = Vec::with_capacity(npoints);
    #[allow(clippy::uninit_vec)]
    unsafe {
        ret.set_len(npoints)
    };

    let pool = da_pool();
    let ncpus = pool.max_count();
    if ncpus < 2 || npoints < 768 {
        let p: [*const blst_p1; 2] = [&points[0].0, ptr::null()];
        unsafe { blst_p1s_to_affine(&mut ret[0], &p[0], npoints) };
        return ret;
    }

    let mut nslices = (npoints + 511) / 512;
    nslices = core::cmp::min(nslices, ncpus);
    let wg = Arc::new((Barrier::new(2), AtomicUsize::new(nslices)));

    let (mut delta, mut rem) = (npoints / nslices + 1, Wrapping(npoints % nslices));
    let mut x = 0usize;
    while x < npoints {
        let out = &mut ret[x];
        let inp = &points[x].0;

        delta -= (rem == Wrapping(0)) as usize;
        rem -= Wrapping(1);
        x += delta;

        let wg = wg.clone();
        pool.joined_execute(move || {
            let p: [*const blst_p1; 2] = [inp, ptr::null()];
            unsafe { blst_p1s_to_affine(out, &p[0], delta) };
            if wg.1.fetch_sub(1, Ordering::AcqRel) == 1 {
                wg.0.wait();
            }
        });
    }
    wg.0.wait();

    ret
}

pub fn multiply(
    points: &[blst_p1_affine],
    scalars: &[u8],
    nbits: usize,
    window: usize,
    pool: ThreadPool,
) -> blst_p1 {
    let ncpus = pool.max_count();
    let npoints = points.len();
    let nbytes = (nbits + 7) / 8;
    let (nx, ny, window) = breakdown(nbits, window, ncpus);

    // |grid[]| holds "coordinates" and place for result
    let mut grid: Vec<(Tile, Cell<blst_p1>)> = Vec::with_capacity(nx * ny);
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
        grid[total].0.dy = nbits - y;
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
            let mut scratch = vec![P1XYZZ::default(); 1usize << (window - 1)];

            loop {
                let work = counter.fetch_add(1, Ordering::Relaxed);
                if work >= total {
                    break;
                }
                let x = grid[work].0.x;
                let y = grid[work].0.y;
                let dx = grid[work].0.dx;

                pippenger_tile_pub(
                    unsafe { grid[work].1.as_ptr().as_mut() }.unwrap(),
                    &points[x..(x + dx)],
                    dx,
                    &scalars[x * nbytes..],
                    nbits,
                    &mut scratch,
                    y,
                    window,
                );
                if row_sync[y / window].fetch_add(1, Ordering::AcqRel) == nx - 1 {
                    tx.send(y).expect("disaster");
                }
            }
        });
    }

    let mut ret = <blst_p1>::default();
    let mut rows = vec![false; ny];
    let mut row = 0usize;
    for _ in 0..ny {
        let mut y = rx.recv().unwrap();
        rows[y / window] = true;
        while grid[row].0.y == y {
            while row < total && grid[row].0.y == y {
                unsafe {
                    blst_p1_add_or_double(&mut ret, &ret, grid[row].1.as_ptr() as *const blst_p1);
                }
                row += 1;
            }
            if y == 0 {
                break;
            }
            for _ in 0..window {
                unsafe { blst_p1_double(&mut ret, &ret) };
            }
            y -= window;
            if !rows[y / window] {
                break;
            }
        }
    }
    ret
}

fn num_bits(l: usize) -> usize {
    8 * core::mem::size_of_val(&l) - l.leading_zeros() as usize
}

fn breakdown(nbits: usize, window: usize, ncpus: usize) -> (usize, usize, usize) {
    let mut nx: usize;
    let mut wnd: usize;

    if nbits > window * ncpus {
        nx = 1;
        wnd = num_bits(ncpus / 4);
        if (window + wnd) > 18 {
            wnd = window - wnd;
        } else {
            wnd = (nbits / window + ncpus - 1) / ncpus;
            if (nbits / (window + 1) + ncpus - 1) / ncpus < wnd {
                wnd = window + 1;
            } else {
                wnd = window;
            }
        }
    } else {
        nx = 2;
        wnd = window - 2;
        while (nbits / wnd + 1) * nx < ncpus {
            nx += 1;
            wnd = window - num_bits(3 * nx / 2);
        }
        nx -= 1;
        wnd = window - num_bits(3 * nx / 2);
    }
    let ny = nbits / wnd + 1;
    wnd = nbits / ny + 1;

    (nx, ny, wnd)
}
