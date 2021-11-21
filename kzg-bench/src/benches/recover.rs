use criterion::Criterion;
use rand::Rng;
use kzg::{Fr, FFTSettings, FFTFr, Poly, PolyRecover};
use std::convert::TryInto;

pub fn bench_recover<
    TFr: Fr,
    TFTTSettings: FFTSettings<TFr> + FFTFr<TFr>,
    TPoly: Poly<TFr>,
    TPolyRecover: PolyRecover<TFr, TPoly, TFTTSettings>
>(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    for scale in 5..16 {
        let fs = TFTTSettings::new(scale).unwrap();
        let max_width: usize = fs.get_max_width();

        let mut poly = vec![TFr::zero(); max_width];

        for i in 0..(max_width / 2) {
            poly[i] = TFr::from_u64(i.try_into().unwrap());
        }

        let data = fs.fft_fr(&poly, false).unwrap();
        let mut samples: Vec<Option<TFr>> = vec![];

        for i in 0..max_width {
            samples.push(Some(data[i].clone()));
        }

        for _ in 0..(max_width / 2) {
            let mut j: usize = rng.gen::<usize>() % max_width;
            while samples[j].is_none() {
                j = rng.gen::<usize>() % max_width;
            }
            samples[j] = None;
        }

        let id = format!("bench_recover scale: '{}'", scale);
        c.bench_function(&id, |b| b.iter(|| {
            TPolyRecover::recover_poly_from_samples(&samples, &fs).unwrap();
        }));
    }
}