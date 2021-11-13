use criterion::Criterion;
use kzg::{FFTFr, FFTSettings, Fr, G1, FFTG1};

pub fn bench_fft_fr<TFr: Fr, TFFTSettings: FFTSettings<TFr> + FFTFr<TFr>>(c: &mut Criterion) {
    for scale in 4..16 {
        let fft_settings = TFFTSettings::new(scale as usize).unwrap();
        let data: Vec<TFr> = vec![TFr::rand(); fft_settings.get_max_width()];
        let id = format!("bench_fft_fr scale: '{}'", scale);
        c.bench_function(&id, |b| b.iter(|| fft_settings.fft_fr(&data, false)));
    }
}

pub fn bench_fft_g1<TFr: Fr, TG1: G1<TFr>, TFFTSettings: FFTSettings<TFr> + FFTG1<TFr, TG1>>(c: &mut Criterion) {
    for scale in 4..8 {
        let fft_settings = TFFTSettings::new(scale as usize).unwrap();
        let data: Vec<TG1> = vec![TG1::rand(); fft_settings.get_max_width()];
        let id = format!("bench_fft_g1 scale: '{}'", scale);
        c.bench_function(&id, |b| b.iter(|| fft_settings.fft_g1(&data, false)));
    }
}
