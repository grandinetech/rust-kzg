use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::lincomb::bench_g1_lincomb;
use rust_kzg_constantine::kzg_proofs::g1_linear_combination;
use rust_kzg_constantine::types::fp::CtFp;
use rust_kzg_constantine::types::fr::CtFr;
use rust_kzg_constantine::types::g1::{CtG1, CtG1Affine, CtG1ProjAddAffine};

fn bench_g1_lincomb_(c: &mut Criterion) {
    bench_g1_lincomb::<CtFr, CtG1, CtFp, CtG1Affine, CtG1ProjAddAffine>(c, &g1_linear_combination);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_g1_lincomb_
}

criterion_main!(benches);
