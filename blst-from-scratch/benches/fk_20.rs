use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fk20::{fk_single_da, fk_single_da_optimized};
use blst_from_scratch::kzg_types::{FsFFTSettings, FsKZGSettings, FsFK20SingleSettings, FsFr, FsG1, FsG2, FsPoly};
use blst_from_scratch::utils::generate_trusted_setup;

fn fk_single_da_(c: &mut Criterion) {
    fk_single_da::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings, FsFK20SingleSettings>(c, &generate_trusted_setup)
}

fn fk_single_da_optimized_(c: &mut Criterion) {
    fk_single_da_optimized::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings, FsFK20SingleSettings>(c, &generate_trusted_setup)
}

criterion_group!(benches, fk_single_da_, fk_single_da_optimized_);
criterion_main!(benches);