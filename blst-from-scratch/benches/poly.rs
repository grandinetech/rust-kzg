use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::poly::{bench_new_poly_div};
use blst_from_scratch::kzg_types::{FsFr, FsPoly};

fn bench_new_poly_div_(c: &mut Criterion) {
    bench_new_poly_div::<FsFr, FsPoly>(c);
}

criterion_group!(benches, bench_new_poly_div_);
criterion_main!(benches);
