use criterion::Criterion;
use kzg::{FFTFr, FFTG1, FFTSettings, Fr, G1};

pub fn bench_kzg<TFr: Fr, TFFTSettings: FFTSettings<TFr> + FFTFr<TFr>>(c: &mut Criterion) {
    for scale in 1..15 {
        let mut fft_settings = TFFTSettings::new(scale as usize).unwrap();
        //Should exist kzg settings
        //let mut kzg_settings;

        //Copied some kind of method from C
        //generate_trusted_setup(s1, s2, &secret, fs.max_width);

        let mut poly = TPoly::new(fft_settings.get_max_width()).unwrap();
        for i in 0..fft_settings.get_max_width() {
            p.set_coeff_at(i, Fr::rand());
        }
        let id = format!("bench_kzg scale: '{}'", scale);
        //Someting with commiting poly
        //c.bench_function(&id, |b| b.iter(|| commit_to_poly(&commitment, poly, kzg_settings)));
        fft_settings.destroy();
    }
}
