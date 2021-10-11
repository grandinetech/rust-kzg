use criterion::{criterion_group, criterion_main, Criterion};

fn bench_poly_division_in_finite_field(c: &mut Criterion) {
    c.bench_function(
        "poly_division_in_finite_field",
        |b| b.iter(|| {
            kzg::poly::poly_division_in_finite_field(8)
        })
    );
}

criterion_group!(benches, bench_poly_division_in_finite_field);
criterion_main!(benches);
