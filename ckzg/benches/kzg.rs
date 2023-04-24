use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::kzg::commit_to_poly;

use ckzg::consts::{BlstP1, BlstP2};
use ckzg::fftsettings::KzgFFTSettings;
use ckzg::finite::BlstFr;
use ckzg::kzgsettings::{generate_trusted_setup, KzgKZGSettings};
use ckzg::poly::KzgPoly;

fn commit_to_poly_(c: &mut Criterion) {
    commit_to_poly::<BlstFr, BlstP1, BlstP2, KzgPoly, KzgFFTSettings, KzgKZGSettings>(
        c,
        &generate_trusted_setup,
    );
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = commit_to_poly_
}

criterion_main!(benches);
