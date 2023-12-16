use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::lincomb::bench_g1_lincomb;
use rust_kzg_blst::kzg_prooCt::g1_linear_combination;
use rust_kzg_blst::types::fr::CtFr;
use rust_kzg_blst::types::g1::CtG1;

fn bench_g1_lincomb_(c: &mut Criterion) {
    bench_g1_lincomb::<CtFr, CtG1>(c, &g1_linear_combination);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_g1_lincomb_
}

criterion_main!(benches);
