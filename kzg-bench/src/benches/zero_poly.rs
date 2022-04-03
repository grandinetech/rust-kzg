use criterion::Criterion;
use rand::seq::SliceRandom;
use rand::thread_rng;
use kzg::{FFTSettings, Fr, Poly, ZeroPoly};

const BENCH_SCALE: usize = 15;

pub fn bench_zero_poly<
    TFr: Fr,
    TFFTSettings: FFTSettings<TFr> + ZeroPoly<TFr, TPoly>,
    TPoly: Poly<TFr>,
>(
    c: &mut Criterion,
) {
    let fs = TFFTSettings::new(BENCH_SCALE).unwrap();
    let size = fs.get_max_width();
    let mut missing = vec![usize::default(); size];
    for i in 0..size {
        missing[i] = i;
    }
    let mut rng = thread_rng();
    missing.shuffle(&mut rng);
    let id = format!("bench_zero_poly scale: '{}'", BENCH_SCALE);
    c.bench_function(&id, |b| b.iter(|| {
        // Half missing leaves enough FFT computation space
        fs.zero_poly_via_multiplication(size, &missing[0..(size / 2)])
    }));
}
