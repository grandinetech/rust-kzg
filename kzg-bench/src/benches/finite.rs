// use criterion::{black_box, criterion_group, criterion_main, Criterion};
// use kzg::Fr;
// use test::black_box;
//
// fn criterion_benchmark(c: &mut Criterion) {
//     c.bench_function("add_fr", |b| b.iter(||
//         kzg::finite::add_fr(black_box(kzg::Fr::default()),
//                             black_box(kzg::Fr::default()))));
// }
//
// criterion_group!(benches, criterion_benchmark);
// criterion_main!(benches);
