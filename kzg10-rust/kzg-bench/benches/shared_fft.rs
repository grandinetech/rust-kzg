use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fft::bench_fft_fr;
use mcl_rust::data_types::fr::Fr;
use mcl_rust::fk20_fft::FFTSettings;
use mcl_rust::mcl_methods::init;
use mcl_rust::CurveType;

fn bench_fft_fr_(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_fft_fr::<Fr, FFTSettings>(c);
}

criterion_group!(benches, bench_fft_fr_);
criterion_main!(benches);