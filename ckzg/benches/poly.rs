use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::poly::bench_new_poly_div;
use ckzg::finite::BlstFr;
use ckzg::poly::KzgPoly;

fn bench_new_poly_div_(c: &mut Criterion) {
    bench_new_poly_div::<BlstFr, KzgPoly>(c);
}

criterion_group!(benches, bench_new_poly_div_);
criterion_main!(benches);
