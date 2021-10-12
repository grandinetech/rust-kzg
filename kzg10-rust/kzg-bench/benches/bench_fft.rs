use mcl_rust::*;
use criterion::{criterion_group, criterion_main, Criterion};
use mcl_rust::fr::Fr;
use mcl_rust::g1::G1;
use mcl_rust::implem::Curve;
use mcl_rust::mlc_methods::init;
use mcl_rust::implem::FFTSettings;

fn bench_fft_fr(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    for scale in 4..16 {
        _fft_fr(scale, c);
    }
}

fn _fft_fr(scale: u8, c: &mut Criterion) {
    let settings = FFTSettings::new(scale);
    let mut data: Vec<Fr> = (0..(settings.max_width >> 1)).map(|_| Fr::random()).collect();
    let id = format!("bench_fft_fr scale: '{}'", scale);
    c.bench_function(&id, |b| b.iter(|| settings.fft(&mut data, false)));
}

fn bench_fft_g1(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    for scale in 4..16 {
        _fft_g1(scale, c);
    }
}

fn _fft_g1(scale: u8, c: &mut Criterion) {
    let settings = FFTSettings::new(scale);
    let curve = Curve::new(&Fr::random(), 2);

    let data: Vec<G1> = (0..(settings.max_width >> 1)).map(|_| &curve.g1_gen * &Fr::random()).collect();
    let id = format!("bench_fft_g1 scale: '{}'", scale);
    c.bench_function(&id, |b| b.iter(|| settings.fft_g1(&data)));
}



criterion_group!(benches, bench_fft_fr, bench_fft_g1);
criterion_main!(benches);
