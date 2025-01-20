use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::poly::bench_new_poly_div;
use rust_kzg_mcl::types::fr::MclFr;
use rust_kzg_mcl::types::poly::MclPoly;

fn bench_new_poly_div_(c: &mut Criterion) {
    bench_new_poly_div::<MclFr, MclPoly>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_new_poly_div_
}

criterion_main!(benches);
