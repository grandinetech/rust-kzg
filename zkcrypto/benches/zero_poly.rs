use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::zero_poly::bench_zero_poly;

use rust_kzg_arkworks::kzg_proofs::FFTSettings;
use rust_kzg_arkworks::kzg_types::ArkFr;
use rust_kzg_arkworks::utils::PolyData;

fn bench_zero_poly_(c: &mut Criterion) {
    bench_zero_poly::<ArkFr, FFTSettings, PolyData>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_zero_poly_
}

criterion_main!(benches);
