use criterion::{Criterion, criterion_group, criterion_main};
use kzg::Fr;
use kzg_from_scratch::fft_fr::fft_fr;
use kzg_from_scratch::kzg_types::{create_fr_rand, FsFFTSettings};

// fn bench_fft_fr(c: &mut Criterion) {
//     for scale in 4..16 {
//         _fft_fr(scale, c);
//     }
// }
//
// fn _fft_fr(scale: u8, c: &mut Criterion) {
//     let fft_settings = FsFFTSettings::from_scale(scale as usize).unwrap();
//     let mut data: Vec<Fr> = vec![create_fr_rand(); fft_settings.max_width];
//     let mut cache: Vec<Fr> = vec![Fr::default(); fft_settings.max_width];
//     let id = format!("bench_fft_fr scale: '{}'", scale);
//     c.bench_function(&id, |b| b.iter(|| fft_fr(&mut cache, &data, false, &fft_settings)));
// }

/*fn bench_fft_g1(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    for scale in 4..16 {
        _fft_g1(scale, c);
    }
}

fn _fft_g1(scale: u8, c: &mut Criterion) {
    let settings = FFTSettings::new(scale);
    let curve = Curve::new(&Fr::random(), 2);

    let data: Vec<G1> = (0..(settings.max_width >> 1)).map(|_| &curve.g1_gen * &Fr::random()).collect();
    let id = format!("bench_fft_g1 scale: '{}'", scale);
    c.bench_function(&id, |b| b.iter(|| settings.fft_g1(&data)));
}
*/

//criterion_group!(benches, bench_fft_fr, bench_fft_g1);
// criterion_group!(benches, bench_fft_fr);
// criterion_main!(benches);