use kzg::{Fr, Poly, FFTSettings, FFTFr};
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use std::convert::TryInto;

fn shuffle(a: &mut [usize], n: usize) {
    let mut i: u64 = n as u64;
    let mut j: usize;
    let mut tmp: usize;
    
    let mut rng = StdRng::seed_from_u64(0);
    while i > 0 {
        j = (rng.next_u64() % i) as usize;
        i -= 1;
        tmp = a[j];
        a[j] = a[i as usize];
        a[i as usize] = tmp;
    }
}

fn random_missing<TFr: Fr>(with_missing: &mut [TFr], data: &[TFr], len_data: usize, known: usize) {
    let mut missing_idx = Vec::default();
    for i in 0..len_data {
        missing_idx.push(i);
        with_missing[i] = data[i].clone();
    }

    shuffle(&mut missing_idx, len_data);
    for i in 0..(len_data - known) {
        with_missing[missing_idx[i]] = TFr::null();
    }
}

pub fn recover_simple<TFr: Fr, TFFTSettings: FFTSettings<TFr> + FFTFr<TFr>, TPoly: Poly<TFr>>(
    recover_poly_from_samples: &dyn Fn(&[TFr], usize, &TFFTSettings) -> Result<Vec<TFr>, String>
) {
    let fs_query = TFFTSettings::new(2);
    assert!(fs_query.is_ok());

    let fs: TFFTSettings = fs_query.unwrap();
    let poly_query = TPoly::new(fs.get_max_width());
    assert!(poly_query.is_ok());
    let mut poly = poly_query.unwrap();

    for i in 0..(fs.get_max_width() / 2) {
        poly.set_coeff_at(i, &TFr::from_u64(i.try_into().unwrap()));
    }

    for i in (fs.get_max_width() / 2)..fs.get_max_width() {
        poly.set_coeff_at(i, &TFr::zero());
    }

    let data_query = fs.fft_fr(&poly.get_coeffs(), false);
    assert!(data_query.is_ok());
    let data = data_query.unwrap();

    let mut sample = vec![TFr::default(); fs.get_max_width()]; // Vec::default(); //[TFr; fs.get_max_width()];
    sample.push(data[0].clone());
    sample.push(TFr::null());
    sample.push(TFr::null());
    sample.push(data[3].clone());

    let recovered_query = recover_poly_from_samples(&sample, fs.get_max_width(), &fs);
    assert!(recovered_query.is_ok());
    let recovered = recovered_query.unwrap();

    for i in 0..fs.get_max_width() {
        assert!(recovered[i].equals(&data[i]));
    }

    let back_query = fs.fft_fr(&recovered, true);
    assert!(back_query.is_ok());
    let back = back_query.unwrap();

    for i in 0..(fs.get_max_width() / 2) {
        assert!(back[i].equals(&poly.get_coeff_at(i)));
    }

    for i in (fs.get_max_width() / 2)..fs.get_max_width() {
        assert!(back[i].is_zero());
    }
}

pub fn recover_random<TFr: Fr, TFFTSettings: FFTSettings<TFr> + FFTFr<TFr>, TPoly: Poly<TFr>>(
    recover_poly_from_samples: &dyn Fn(&[TFr], usize, &TFFTSettings) -> Result<Vec<TFr>, String>
) {
    let fs_query = TFFTSettings::new(12);
    assert!(fs_query.is_ok());

    let fs: TFFTSettings = fs_query.unwrap();
    // let mut poly = TPoly::default();
    let poly_query = TPoly::new(fs.get_max_width());
    assert!(poly_query.is_ok());
    let mut poly = poly_query.unwrap();

    for i in 0..(fs.get_max_width() / 2) {
        poly.set_coeff_at(i, &TFr::from_u64(i.try_into().unwrap()));
    }

    for i in (fs.get_max_width() / 2)..fs.get_max_width() {
        poly.set_coeff_at(i, &TFr::zero());
    }

    let data_query = fs.fft_fr(&poly.get_coeffs(), false);
    assert!(data_query.is_ok());
    let data = data_query.unwrap();
    
    let mut samples = vec![TFr::default(); fs.get_max_width()]; // std::vec![TFr; fs.get_max_width()];

    for i in 0..10 {
        let known_ratio = 0.5 + (i as f32) * 0.05;
        let known: usize = ((fs.get_max_width() as f32) * known_ratio) as usize;

        for i in 0..4 {
            random_missing(&mut samples, &data, fs.get_max_width(), known);

            let recovered_query = recover_poly_from_samples(&samples, fs.get_max_width(), &fs);
            assert!(recovered_query.is_ok());
            let recovered = recovered_query.unwrap();

            for j in 0..fs.get_max_width() {
                assert!(data[i].equals(&recovered[i]));
            }

            let back_query = fs.fft_fr(&recovered, true);
            assert!(back_query.is_ok());
            let back = back_query.unwrap();

            for i in 0..(fs.get_max_width() / 2) {
                assert!(back[i].equals(&poly.get_coeff_at(i)));
            }

            for i in (fs.get_max_width() / 2)..fs.get_max_width() {
                assert!(back[i].is_zero());
            }
        }
    }
}
