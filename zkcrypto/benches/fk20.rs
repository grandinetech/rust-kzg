use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fk20::{bench_fk_multi_da, bench_fk_single_da};

use rust_kzg_zkcrypto::fftsettings::ZkFFTSettings;
use rust_kzg_zkcrypto::fk20::{ZkFK20MultiSettings, ZkFK20SingleSettings};
use rust_kzg_zkcrypto::kzg_proofs::{generate_trusted_setup, KZGSettings};
use rust_kzg_zkcrypto::kzg_types::{ZkG1Projective, ZkG2Projective};
use rust_kzg_zkcrypto::poly::ZPoly;
use rust_kzg_zkcrypto::zkfr::blsScalar;

fn bench_fk_single_da_(c: &mut Criterion) {
    bench_fk_single_da::<
        blsScalar,
        ZkG1Projective,
        ZkG2Projective,
        ZPoly,
        ZkFFTSettings,
        KZGSettings,
        ZkFK20SingleSettings,
    >(c, &generate_trusted_setup)
}

fn bench_fk_multi_da_(c: &mut Criterion) {
    bench_fk_multi_da::<
        blsScalar,
        ZkG1Projective,
        ZkG2Projective,
        ZPoly,
        ZkFFTSettings,
        KZGSettings,
        ZkFK20MultiSettings,
    >(c, &generate_trusted_setup)
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_fk_single_da_, bench_fk_multi_da_
}

criterion_main!(benches);
