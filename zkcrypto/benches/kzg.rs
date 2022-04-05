use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::kzg::kzg_proof;

use zkcrypto::fftsettings::ZkFFTSettings;
use zkcrypto::zkfr::blsScalar;
use zkcrypto::kzg_types::{ZkG1Projective, ZkG2Projective};
use zkcrypto::kzg_proofs::{KZGSettings, generate_trusted_setup};
use zkcrypto::poly::ZPoly;

fn kzg_proof_(c: &mut Criterion) {
    kzg_proof::<blsScalar, ZkG1Projective, ZkG2Projective, ZPoly, ZkFFTSettings, KZGSettings>(c, &generate_trusted_setup)
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(2);
    targets = kzg_proof_
}

criterion_main!(benches);