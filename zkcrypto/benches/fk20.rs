use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fk20::{fk_single_da, fk_multi_da};

use zkcrypto::fftsettings::ZkFFTSettings;
use zkcrypto::fk20::{ZkFK20MultiSettings, ZkFK20SingleSettings};
use zkcrypto::zkfr::blsScalar;
use zkcrypto::kzg_types::{ZkG1Projective, ZkG2Projective};
use zkcrypto::kzg_proofs::{KZGSettings, generate_trusted_setup};
use zkcrypto::poly::ZPoly;

fn fk_single_da_(c: &mut Criterion) {
    fk_single_da::<
        blsScalar,
        ZkG1Projective,
        ZkG2Projective,
        ZPoly,
        ZkFFTSettings,
        KZGSettings,
        ZkFK20SingleSettings
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
    config = Criterion::default().sample_size(2);
    targets = fk_single_da_, fk_multi_da_
}

criterion_main!(benches);