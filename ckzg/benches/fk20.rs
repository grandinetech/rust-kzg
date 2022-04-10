use criterion::{Criterion, criterion_group, criterion_main};
use kzg_bench::benches::fk20::*;

use ckzg::consts::{BlstP1, BlstP2};
use ckzg::fftsettings::KzgFFTSettings;
use ckzg::fk20settings::{KzgFK20MultiSettings, KzgFK20SingleSettings};
use ckzg::kzgsettings::{KzgKZGSettings, generate_trusted_setup};
use ckzg::finite::BlstFr;
use ckzg::poly::KzgPoly;

fn fk_single_da_(c: &mut Criterion) {
    fk_single_da::<BlstFr, BlstP1, BlstP2, KzgPoly, KzgFFTSettings, KzgKZGSettings, KzgFK20SingleSettings>(c, &generate_trusted_setup);
}

fn fk_multi_da_(c: &mut Criterion) {
    fk_multi_da::<BlstFr, BlstP1, BlstP2, KzgPoly, KzgFFTSettings, KzgKZGSettings, KzgFK20MultiSettings>(c, &generate_trusted_setup);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(2);
    targets = fk_single_da_, fk_multi_da_
}

criterion_main!(benches);
