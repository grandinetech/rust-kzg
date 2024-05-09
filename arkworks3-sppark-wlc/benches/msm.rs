// Copyright Supranational LLC
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main, Criterion};

use ark_bls12_381::G1Affine;
use ark_ff::BigInteger256;

use std::str::FromStr;

use blst_msm::*;

fn criterion_benchmark(c: &mut Criterion) {
    let bench_npow = std::env::var("BENCH_NPOW").unwrap_or("20".to_string());
    let npoints_npow = i32::from_str(&bench_npow).unwrap();

    let batches = 16;
    let (points, scalars) =
        util::generate_points_scalars::<G1Affine>(1usize << npoints_npow, batches);
    let mut context = multi_scalar_mult_init(points.as_slice());

    let mut group = c.benchmark_group("CUDA");
    group.sample_size(10);

    let name = format!("2**{}x{}", npoints_npow, batches);
    group.bench_function(name, |b| {
        b.iter(|| {
            let _ = multi_scalar_mult(&mut context, &points.as_slice(), unsafe {
                std::mem::transmute::<&[_], &[BigInteger256]>(
                    scalars.as_slice(),
                )
            });
        })
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
