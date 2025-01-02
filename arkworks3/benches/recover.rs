use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::recover::bench_recover;

use rust_kzg_arkworks3::kzg_proofs::LFFTSettings as FFTSettings;
use rust_kzg_arkworks3::kzg_types::ArkFr;
use rust_kzg_arkworks3::utils::PolyData;

fn bench_recover_(c: &mut Criterion) {
    bench_recover::<ArkFr, FFTSettings, PolyData, PolyData>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_recover_
}

criterion_main!(benches);
