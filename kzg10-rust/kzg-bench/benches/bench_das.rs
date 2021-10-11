use mcl_rust::*;
use criterion::{criterion_group, criterion_main, Criterion};
use mcl_rust::mlc_methods::init;
use mcl_rust::implem::FFTSettings;
use mcl_rust::fr::Fr;

fn bench_das_fft_extension(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    for scale in 4..16 {
        _set_up_and_bench(scale, c);
    }
}

fn _set_up_and_bench(scale: u8, c: &mut Criterion) {
    let settings = FFTSettings::new(scale);
    let mut data: Vec<Fr> = (0..(settings.max_width >> 1)).map(|_| Fr::random()).collect();
    let id = format!("bench_das_fft_extension scale: '{}'", scale);
    c.bench_function(&id, |b| b.iter(|| settings.das_fft_extension(&mut data)));
}

criterion_group!(benches, bench_das_fft_extension);
criterion_main!(benches);