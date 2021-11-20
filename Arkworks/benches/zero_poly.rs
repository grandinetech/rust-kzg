use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::zero_poly::bench_zero_poly;

use arkworks::kzg_types::FsFr;
use arkworks::kzg_proofs::FFTSettings;
use arkworks::utils::PolyData;

fn bench_zero_poly_(c: &mut Criterion) {
    bench_zero_poly::<FsFr, FFTSettings, PolyData>(c);
}

criterion_group!(benches, bench_zero_poly_);
criterion_main!(benches);