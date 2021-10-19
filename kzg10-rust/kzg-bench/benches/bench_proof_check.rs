use mcl_rust::*;
use criterion::{criterion_group, criterion_main, Criterion};
use mcl_rust::fr::Fr;
use mcl_rust::implem::Curve;
use mcl_rust::implem::Polynomial;
use mcl_rust::mcl_methods::init;

fn bench_simple_proof_check(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    let coefficients = vec![1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13];
    let poly = Polynomial::from_i32(&coefficients);
    let secret = Fr::from_str("1927409816240961209460912649124", 10);
    let curve = Curve::new(&secret.unwrap(), poly.order());
    let x = Fr::from_int(17);
    let y = poly.eval_at(&x);
    let proof = poly.gen_proof_at(&curve.g1_points, &x);
    let commitment = poly.commit(&curve.g1_points);

    c.bench_function("bench_simple_proof_check", |b| b.iter(|| curve.is_proof_valid(&commitment, &proof, &x, &y)));
}

fn bench_large_proof_check(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    let poly = Polynomial::random(2048);
    let secret = Fr::from_str("1927409816240961209460912649124", 10);
    let curve = Curve::new(&secret.unwrap(), poly.order());
    let x = Fr::from_int(17);
    let y = poly.eval_at(&x);
    let proof = poly.gen_proof_at(&curve.g1_points, &x);
    let commitment = poly.commit(&curve.g1_points);

    c.bench_function("bench_large_proof_check", |b| b.iter(|| curve.is_proof_valid(&commitment, &proof, &x, &y)));
}

criterion_group!(benches, bench_simple_proof_check, bench_large_proof_check);
criterion_main!(benches);