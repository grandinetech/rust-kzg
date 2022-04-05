use criterion::{Criterion, criterion_group, criterion_main};
use kzg_bench::benches::kzg::kzg_proof;

use ckzg::consts::{BlstP1, BlstP2};
use ckzg::fftsettings::KzgFFTSettings;
use ckzg::kzgsettings::{generate_trusted_setup, KzgKZGSettings};
use ckzg::poly::KzgPoly;
use ckzg::finite::BlstFr;

fn kzg_proof_(c: &mut Criterion) {
    kzg_proof::<BlstFr, BlstP1, BlstP2, KzgPoly, KzgFFTSettings, KzgKZGSettings>(c, &generate_trusted_setup);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(2);
    targets = kzg_proof_
}

criterion_main!(benches);
