use criterion::Criterion;
use kzg::{FFTFr, FFTG1, FFTSettings, Fr, G1};

pub fn bench_kzg<TFr: Fr, TFFTSettings: FFTSettings<TFr> + FFTFr<TFr>>(c: &mut Criterion) {
    for scale in 1..15 {
        let mut fft_settings = TFFTSettings::new(scale as usize).unwrap();
        //Should exist kzg settings
        //let mut kzg_settings;

        //Start of Trusted setup

        //Need to find in mlc fr_from_scale method
        //let s = Fr::from_scale()
        //fr_from_scalar(s, secret);
        let s_pow = TFr::one();

        for i in 0..fft_settings.get_max_width() {
            mclBnG1_mul(s1 + i, g1_generator, s_pow);//Not universal solution
            mclBnG2_mul(s2 + i, g2_generator, s_pow);//Not universal solution
            mclBnFr_mul(s_pow, s_pow, s);//Not universal solution (Fr::mul exist)
        }
        //End of Trusted setup

        let mut poly = TPoly::new(fft_settings.get_max_width()).unwrap();
        for i in 0..fft_settings.get_max_width() {
            p.set_coeff_at(i, Fr::rand());
        }
        let id = format!("bench_kzg scale: '{}'", scale);
        //Commiting to poly not exist
        //c.bench_function(&id, |b| b.iter(|| commit_to_poly(&commitment, poly, kzg_settings)));
        fft_settings.destroy();
    }
}
