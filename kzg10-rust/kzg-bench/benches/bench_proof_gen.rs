use mcl_rust::*;
use criterion::{criterion_group, criterion_main, Criterion};
use mcl_rust::implem::FK20Matrix;
use mcl_rust::implem::Polynomial;
use mcl_rust::implem::Curve;
use mcl_rust::fr::Fr;
use mcl_rust::mcl_methods::init;

fn bench_simple_proof_gen(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    let coefficients = vec![1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13];
    let poly = Polynomial::from_i32(&coefficients);
    let secret = Fr::from_str("1927409816240961209460912649124", 10);
    let curve = Curve::new(&secret.unwrap(), poly.order());
    let x = Fr::from_int(17);
    c.bench_function("bench_simple_proof_gen", |b| b.iter(|| poly.gen_proof_at(&curve.g1_points, &x)));
}

fn bench_large_proof_gen(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    let poly = Polynomial::random(2048);
    let secret = Fr::from_str("1927409816240961209460912649124", 10);
    let curve = Curve::new(&secret.unwrap(), poly.order());
    let x = Fr::from_int(17);
    c.bench_function("bench_large_proof_gen", |b| b.iter(|| poly.gen_proof_at(&curve.g1_points, &x)));
}

fn bench_random_multi_proof_gen(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    let chunk_len: usize = 16;
    let chunk_count: usize = 32;
    let n = chunk_len * chunk_count;
    let n2 = n << 1;
    let secret = Fr::from_str("1927409816240961209460912649124", 10).unwrap();
    let kzg_curve = Curve::new(&secret, n2);
    let matrix = FK20Matrix::new(kzg_curve, n2, chunk_len, 10);
    let polynomial = Polynomial::random(n);
    c.bench_function("bench_random_multi_proof_gen", |b| b.iter(|| matrix.dau_using_fk20_multi(&polynomial)));
}

criterion_group!(benches, bench_simple_proof_gen, bench_large_proof_gen, bench_random_multi_proof_gen);
criterion_main!(benches);