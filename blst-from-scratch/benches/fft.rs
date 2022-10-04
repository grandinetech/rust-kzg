use blst_from_scratch::types::fft_settings::FsFFTSettings;
use blst_from_scratch::types::fr::FsFr;
use blst_from_scratch::types::g1::FsG1;
use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fft::{bench_fft_fr, bench_fft_g1};

fn bench_fft_fr_(c: &mut Criterion) {
    bench_fft_fr::<FsFr, FsFFTSettings>(c);
}

fn bench_fft_g1_(c: &mut Criterion) {
    bench_fft_g1::<FsFr, FsG1, FsFFTSettings>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_fft_fr_, bench_fft_g1_
}

criterion_main!(benches);
