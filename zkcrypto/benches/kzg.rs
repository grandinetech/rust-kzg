use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::kzg::kzg_proof;

use zkcrypto::fftsettings::ZkFFTSettings;
use zkcrypto::kzg_proofs::{generate_trusted_setup, KZGSettings};
use zkcrypto::kzg_types::{ZkG1Projective, ZkG2Projective};
use zkcrypto::poly::ZPoly;
use zkcrypto::zkfr::blsScalar;

fn kzg_proof_(c: &mut Criterion) {
    kzg_proof::<blsScalar, ZkG1Projective, ZkG2Projective, ZPoly, ZkFFTSettings, KZGSettings>(
        c,
        &generate_trusted_setup,
    )
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = kzg_proof_
}

criterion_main!(benches);
