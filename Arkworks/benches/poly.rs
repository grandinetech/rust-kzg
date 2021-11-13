use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::poly::bench_new_poly_div;
use arkworks::kzg_types::FsFr;
use arkworks::utils::PolyData;


fn bench_new_poly_div_(c: &mut Criterion) {
    bench_new_poly_div::<FsFr, PolyData>(c);
}

criterion_group!(benches, bench_new_poly_div_);
criterion_main!(benches);
