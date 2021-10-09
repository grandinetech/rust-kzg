use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kzg::Fr;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("add_fr", |b| b.iter(|| Fr::add(black_box(Fr::default()), black_box(Fr::default()))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
