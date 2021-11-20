use criterion::Criterion;
use kzg::{FFTSettings, Fr, ZeroPoly, Poly};

pub fn bench_zero_poly<TFr: Fr, TFFTSettings: FFTSettings<TFr> + ZeroPoly<TFr, TPoly>, TPoly: Poly<TFr>>(c: &mut Criterion) {
	for scale in 5..16 {
		let fs = TFFTSettings::new(scale).unwrap();
		let size = TFFTSettings::get_max_width(&fs);
		let mut missing = vec![usize::default(); size];
		for i in 0..size {
			missing[i] = i;
		}
		let id = format!("bench_zero_poly scale: '{}'", scale);
		c.bench_function(&id, |b| b.iter(|| TFFTSettings::zero_poly_via_multiplication(&fs, size, &missing)));
	}
}