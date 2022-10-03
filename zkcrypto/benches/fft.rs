use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fft::{bench_fft_fr, bench_fft_g1};
use zkcrypto::fftsettings::ZkFFTSettings;
use zkcrypto::kzg_types::ZkG1Projective;
use zkcrypto::zkfr::blsScalar;

fn bench_fft_fr_(c: &mut Criterion) {
    bench_fft_fr::<blsScalar, ZkFFTSettings>(c);
}

fn bench_fft_g1_(c: &mut Criterion) {
    bench_fft_g1::<blsScalar, ZkG1Projective, ZkFFTSettings>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_fft_fr_, bench_fft_g1_
}

criterion_main!(benches);
