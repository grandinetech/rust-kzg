use criterion::Criterion;
use kzg::{Fr, FFTSettings, DAS};

pub fn bench_das_extension<TFr: Fr, TFFTSettings: FFTSettings<TFr> + DAS<TFr>>(c: &mut Criterion) {
    for scale in 4..16 {
        let fft_settings = TFFTSettings::new(scale as usize).unwrap();
        let data: Vec<TFr> = vec![TFr::rand(); fft_settings.get_max_width()/2];
        let id = format!("bench_DAS_extension scale: '{}'", scale);
        c.bench_function(&id, |b| b.iter(|| fft_settings.das_fft_extension(&data)));
    }
}