use criterion::{black_box, Criterion};
use kzg::{FFTFr, FFTSettings, Fr, Poly, PolyRecover};
use rand::Rng;
use std::convert::TryInto;

const BENCH_SCALE: usize = 15;

pub fn bench_recover<
    TFr: Fr,
    TFTTSettings: FFTSettings<TFr> + FFTFr<TFr>,
    TPoly: Poly<TFr>,
    TPolyRecover: PolyRecover<TFr, TPoly, TFTTSettings>,
>(
    c: &mut Criterion,
) {
    let mut rng = rand::thread_rng();
    let fs = TFTTSettings::new(BENCH_SCALE).unwrap();
    let max_width: usize = fs.get_max_width();

    let mut poly = vec![TFr::zero(); max_width];

    for (i, p) in poly.iter_mut().enumerate().take(max_width / 2) {
        *p = TFr::from_u64(i.try_into().unwrap());
    }

    let mut samples = fs
        .fft_fr(&poly, false)
        .unwrap()
        .into_iter()
        .map(Some)
        .collect::<Vec<_>>();

    for _ in 0..(max_width / 2) {
        let mut j: usize = rng.gen::<usize>() % max_width;
        while samples[j].is_none() {
            j = rng.gen::<usize>() % max_width;
        }
        samples[j] = None;
    }

    let id = format!("bench_recover scale: '{}'", BENCH_SCALE);
    c.bench_function(&id, |b| {
        b.iter(|| {
            TPolyRecover::recover_poly_from_samples(black_box(&samples), black_box(&fs)).unwrap();
        })
    });
}
