use criterion::Criterion;
use kzg::{FFTSettings, Fr, DAS};

const BENCH_SCALE: usize = 15;

pub fn bench_das_extension<TFr: Fr, TFFTSettings: FFTSettings<TFr> + DAS<TFr>>(c: &mut Criterion) {
    let fft_settings = TFFTSettings::new(BENCH_SCALE).unwrap();
    let data: Vec<TFr> = vec![TFr::rand(); fft_settings.get_max_width() / 2];
    let id = format!("bench_DAS_extension scale: '{}'", BENCH_SCALE);
    c.bench_function(&id, |b| b.iter(|| fft_settings.das_fft_extension(&data)));
}
