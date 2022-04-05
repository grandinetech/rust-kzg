use criterion::Criterion;
use kzg::{FFTFr, FFTSettings, Fr, Poly, PolyRecover};
use rand::Rng;
use std::convert::TryInto;

const BENCH_SCALE: usize = 15;

pub fn bench_recover<
    TFr: 'static + Fr,
    TFTTSettings: 'static + FFTSettings<TFr> + FFTFr<TFr>,
    TPoly: 'static + Poly<TFr>,
    TPolyRecover: 'static + PolyRecover<TFr, TPoly, TFTTSettings>,
>(
    c: &mut Criterion,
) {
    let mut rng = rand::thread_rng();
    let fs = TFTTSettings::new(BENCH_SCALE).unwrap();
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

    let id = format!("bench_recover scale: '{}'", BENCH_SCALE);
    c.bench_function(&id, move |b| b.iter(|| {
        TPolyRecover::recover_poly_from_samples(&samples, &fs).unwrap();
    }));
}
