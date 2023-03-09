use criterion::Criterion;
use kzg::{FFTSettings, Fr, Poly, ZeroPoly};
use rand::seq::SliceRandom;
use rand::thread_rng;

const BENCH_SCALE: usize = 15;

pub fn bench_zero_poly<
    TFr: 'static + Fr,
    TFFTSettings: 'static + FFTSettings<TFr> + ZeroPoly<TFr, TPoly>,
    TPoly: 'static + Poly<TFr>,
>(
    c: &mut Criterion,
) {
    let fs = TFFTSettings::new(BENCH_SCALE).unwrap();
    let size = fs.get_max_width();
    let mut missing = (0..size).collect::<Vec<_>>();
    let mut rng = thread_rng();
    missing.shuffle(&mut rng);
    let id = format!("bench_zero_poly scale: '{}'", BENCH_SCALE);
    c.bench_function(&id, move |b| {
        b.iter(|| {
            // Half missing leaves enough FFT computation space
            fs.zero_poly_via_multiplication(size, &missing[0..(size / 2)])
        })
    });
}
