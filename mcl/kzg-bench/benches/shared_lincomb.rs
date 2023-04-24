use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::lincomb::bench_g1_lincomb;
use mcl_rust::data_types::fr::Fr;
use mcl_rust::data_types::g1::{g1_linear_combination, G1};
use mcl_rust::mcl_methods::init;
use mcl_rust::CurveType;

fn bench_g1_lincomb_(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_g1_lincomb::<Fr, G1>(c, &g1_linear_combination);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_g1_lincomb_
}

criterion_main!(benches);
