use criterion::{criterion_group, criterion_main, Criterion};
use mcl_rust::data_types::fr::Fr;
use mcl_rust::kzg10::Curve;
use mcl_rust::kzg10::Polynomial;
use mcl_rust::mcl_methods::init;
use mcl_rust::*;

fn bench_simple_commit(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    let coefficients = vec![1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13];
    let poly = Polynomial::from_i32(&coefficients);
    let secret = Fr::from_str("1927409816240961209460912649124", 10);
    let curve = Curve::new(&secret.unwrap(), poly.order());
    c.bench_function("bench_simple_commit", move |b| {
        b.iter(|| poly.commit(&curve.g1_points))
    });
}

fn bench_large_commit(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    let poly = Polynomial::random(2048);
    let secret = Fr::from_str("1927409816240961209460912649124", 10);
    let curve = Curve::new(&secret.unwrap(), poly.order());
    c.bench_function("bench_large_commit", move |b| {
        b.iter(|| poly.commit(&curve.g1_points))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_simple_commit, bench_large_commit
}

criterion_main!(benches);
