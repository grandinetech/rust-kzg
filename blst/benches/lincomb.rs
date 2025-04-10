use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::lincomb::bench_g1_lincomb;
use rust_kzg_blst::kzg_proofs::g1_linear_combination;
use rust_kzg_blst::types::fp::FsFp;
use rust_kzg_blst::types::fr::FsFr;
use rust_kzg_blst::types::g1::{FsG1, FsG1Affine, FsG1ProjAddAffine};

fn bench_g1_lincomb_(c: &mut Criterion) {
    bench_g1_lincomb::<FsFr, FsG1, FsFp, FsG1Affine, FsG1ProjAddAffine>(c, &g1_linear_combination);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(100);
    targets = bench_g1_lincomb_
}

criterion_main!(benches);
