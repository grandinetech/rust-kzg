use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::recover::bench_recover;

use rust_kzg_zkcrypto::kzg_proofs::FFTSettings;
use rust_kzg_zkcrypto::kzg_types::ZFr;
use rust_kzg_zkcrypto::poly::PolyData;

fn bench_recover_(c: &mut Criterion) {
    bench_recover::<ZFr, FFTSettings, PolyData, PolyData>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_recover_
}

criterion_main!(benches);
