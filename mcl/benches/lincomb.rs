use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::lincomb::bench_g1_lincomb;
use rust_kzg_mcl::kzg_proofs::g1_linear_combination;
use rust_kzg_mcl::types::fp::MclFp;
use rust_kzg_mcl::types::fr::MclFr;
use rust_kzg_mcl::types::g1::{MclG1, MclG1Affine};

fn bench_g1_lincomb_(c: &mut Criterion) {
    bench_g1_lincomb::<MclFr, MclG1, MclFp, MclG1Affine>(c, &g1_linear_combination);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(100);
    targets = bench_g1_lincomb_
}

criterion_main!(benches);
