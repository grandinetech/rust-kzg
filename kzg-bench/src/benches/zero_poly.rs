use criterion::Criterion;
use kzg::{FFTSettings, Fr, ZeroPoly, Poly};

pub fn bench_zero_poly<TFr: Fr, TFFTSettings: FFTSettings<TFr> + ZeroPoly<TFr, TPoly>, TPoly: Poly<TFr>>(c: &mut Criterion) {
	for scale in 5..16 {
		let fs = TFFTSettings::new(scale).unwrap();
		let mut missing = vec![usize::default(); fs.get_max_width()];
		for i in 0..fs.get_max_width() {
			missing[i] = i;
		}
		let id = format!("bench_zero_poly scale: '{}'", scale);
		c.bench_function(&id, |b| b.iter(|| {
			fs.zero_poly_via_multiplication(fs.get_max_width(), &missing)
		}));
	}
}