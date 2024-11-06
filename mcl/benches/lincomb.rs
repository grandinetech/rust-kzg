use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::lincomb::bench_g1_lincomb;
use rust_kzg_mcl::kzg_proofs::g1_linear_combination;
use rust_kzg_mcl::types::fp::FsFp;
use rust_kzg_mcl::types::fr::FsFr;
use rust_kzg_mcl::types::g1::{FsG1, FsG1Affine};

fn bench_g1_lincomb_(c: &mut Criterion) {
    bench_g1_lincomb::<FsFr, FsG1, FsFp, FsG1Affine>(c, &g1_linear_combination);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(100);
    targets = bench_g1_lincomb_
}

criterion_main!(benches);
