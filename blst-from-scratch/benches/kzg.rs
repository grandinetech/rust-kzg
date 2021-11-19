use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::kzg::{kzg_proof};
use blst_from_scratch::kzg_types::{FsFFTSettings, FsKZGSettings, FsFr, FsG1, FsG2, FsPoly};
use blst_from_scratch::utils::generate_trusted_setup;

fn kzg_proof_(c: &mut Criterion) {
    kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(c, &generate_trusted_setup)
}

criterion_group!(benches, kzg_proof_);
criterion_main!(benches);