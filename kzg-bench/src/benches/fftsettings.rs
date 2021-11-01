// use criterion::{criterion_group, criterion_main, Criterion};
// use kzg::{FFTSettings};

// fn fft_fr(c: &mut Criterion) {
//     c.bench_function(
//         "fft_fr",
//         |b| b.iter(|| {
//             FFTSettings::bench_fft_fr(8)
//         })
//     );
// }

// fn fft_g1(c: &mut Criterion) {
//     c.bench_function(
//         "fft_g1",
//         |b| b.iter(|| {
//             FFTSettings::bench_fft_g1(5)
//         })
//     );
// }

// criterion_group!(benches, fft_fr, fft_g1);
// criterion_main!(benches);
