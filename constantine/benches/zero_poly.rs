use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::zero_poly::bench_zero_poly;

use rust_kzg_zkcrypto::kzg_proofs::FFTSettings;
use rust_kzg_zkcrypto::kzg_types::ZFr;
use rust_kzg_zkcrypto::poly::PolyData;

fn bench_zero_poly_(c: &mut Criterion) {
    bench_zero_poly::<ZFr, FFTSettings, PolyData>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_zero_poly_
}

criterion_main!(benches);
