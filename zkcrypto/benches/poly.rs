use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::poly::bench_new_poly_div;
use zkcrypto::zkfr::blsScalar;
use zkcrypto::poly::ZPoly;

fn bench_new_poly_div_(c: &mut Criterion) {
    bench_new_poly_div::<blsScalar, ZPoly>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(2);
    targets = bench_new_poly_div_
}

criterion_main!(benches);