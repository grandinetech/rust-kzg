use blst_from_scratch::types::fft_settings::FsFFTSettings;
use blst_from_scratch::types::fr::FsFr;
use blst_from_scratch::types::g1::FsG1;
use blst_from_scratch::types::g2::FsG2;
use blst_from_scratch::types::kzg_settings::FsKZGSettings;
use blst_from_scratch::types::poly::FsPoly;
use blst_from_scratch::utils::generate_trusted_setup;
use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::kzg::kzg_proof;

fn kzg_proof_(c: &mut Criterion) {
    kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(c, &generate_trusted_setup)
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = kzg_proof_
}

criterion_main!(benches);
