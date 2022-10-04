use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fk20::{fk_multi_da, fk_single_da};

use zkcrypto::fftsettings::ZkFFTSettings;
use zkcrypto::fk20::{ZkFK20MultiSettings, ZkFK20SingleSettings};
use zkcrypto::kzg_proofs::{generate_trusted_setup, KZGSettings};
use zkcrypto::kzg_types::{ZkG1Projective, ZkG2Projective};
use zkcrypto::poly::ZPoly;
use zkcrypto::zkfr::blsScalar;

fn fk_single_da_(c: &mut Criterion) {
    fk_single_da::<
        blsScalar,
        ZkG1Projective,
        ZkG2Projective,
        ZPoly,
        ZkFFTSettings,
        KZGSettings,
        ZkFK20SingleSettings,
    >(c, &generate_trusted_setup)
}

fn fk_multi_da_(c: &mut Criterion) {
    fk_multi_da::<
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
    targets = fk_single_da_, fk_multi_da_
}

criterion_main!(benches);
