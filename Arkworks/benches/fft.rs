use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fft::{bench_fft_fr, bench_fft_g1};
use arkworks::kzg_proofs::FFTSettings;
use arkworks::kzg_types::{ArkG1, FsFr};

fn bench_fft_fr_(c: &mut Criterion) {
    bench_fft_fr::<FsFr, FFTSettings>(c);
}

fn bench_fft_g1_(c: &mut Criterion) {
    bench_fft_g1::<FsFr, ArkG1, FFTSettings>(c);
}

criterion_group!(benches, bench_fft_fr_, bench_fft_g1_);
criterion_main!(benches);
