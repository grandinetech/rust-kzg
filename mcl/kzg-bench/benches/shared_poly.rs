use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::poly::bench_new_poly_div;
use mcl_rust::data_types::fr::Fr;
use mcl_rust::kzg10::Polynomial;
use mcl_rust::mcl_methods::init;
use mcl_rust::CurveType;

fn bench_new_poly_div_(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_new_poly_div::<Fr, Polynomial>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_new_poly_div_
}

criterion_main!(benches);
