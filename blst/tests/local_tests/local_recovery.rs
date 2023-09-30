use kzg::{FFTFr, FFTSettings, Fr, Poly, PolyRecover};
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

fn random_missing<TFr: Fr>(
    with_missing: &mut [Option<TFr>],
    data: &[TFr],
    len_data: usize,
    known: usize,
) {
    let mut missing_idx = Vec::new();
    for i in 0..len_data {
        missing_idx.push(i);
        with_missing[i] = Some(data[i].clone());
    }

    shuffle(&mut missing_idx, len_data);
    for i in 0..(len_data - known) {
        with_missing[missing_idx[i]] = None;
    }
}

pub fn recover_simple<
    TFr: Fr,
    TFFTSettings: FFTSettings<TFr> + FFTFr<TFr>,
    TPoly: Poly<TFr>,
    TPolyRecover: PolyRecover<TFr, TPoly, TFFTSettings>,
>() {
    let fs_query = TFFTSettings::new(2);
    assert!(fs_query.is_ok());

    let fs: TFFTSettings = fs_query.unwrap();
    let max_width: usize = fs.get_max_width();

    let poly_query = TPoly::new(max_width);
    let mut poly = poly_query;

    for i in 0..(max_width / 2) {
        poly.set_coeff_at(i, &TFr::from_u64(i.try_into().unwrap()));
    }

    for i in (max_width / 2)..max_width {
        poly.set_coeff_at(i, &TFr::zero());
    }

    let data_query = fs.fft_fr(poly.get_coeffs(), false);
    assert!(data_query.is_ok());
    let data = data_query.unwrap();

    let sample: [Option<TFr>; 4] = [Some(data[0].clone()), None, None, Some(data[3].clone())];

    let recovered_query = TPolyRecover::recover_poly_from_samples(&sample, &fs);
    assert!(recovered_query.is_ok());
    let recovered = recovered_query.unwrap();

    for (i, item) in data.iter().enumerate().take(max_width) {
        assert!(item.equals(&recovered.get_coeff_at(i)));
    }

    let mut recovered_vec: Vec<TFr> = Vec::new();

    for i in 0..max_width {
        recovered_vec.push(recovered.get_coeff_at(i));
    }

    let back_query = fs.fft_fr(&recovered_vec, true);
    assert!(back_query.is_ok());
    let back = back_query.unwrap();

    for (i, back_x) in back[..(max_width / 2)].iter().enumerate() {
        assert!(back_x.equals(&poly.get_coeff_at(i)));
    }

    for back_x in back[(max_width / 2)..max_width].iter() {
        assert!(back_x.is_zero());
    }
}

pub fn recover_random<
    TFr: Fr,
    TFFTSettings: FFTSettings<TFr> + FFTFr<TFr>,
    TPoly: Poly<TFr>,
    TPolyRecover: PolyRecover<TFr, TPoly, TFFTSettings>,
>() {
    let fs_query = TFFTSettings::new(12);
    assert!(fs_query.is_ok());

    let fs: TFFTSettings = fs_query.unwrap();
    let max_width: usize = fs.get_max_width();
    // let mut poly = TPoly::default();
    let poly_query = TPoly::new(max_width);
    let mut poly = poly_query;

    for i in 0..(max_width / 2) {
        poly.set_coeff_at(i, &TFr::from_u64(i.try_into().unwrap()));
    }

    for i in (max_width / 2)..max_width {
        poly.set_coeff_at(i, &TFr::zero());
    }

    let data_query = fs.fft_fr(poly.get_coeffs(), false);
    assert!(data_query.is_ok());
    let data = data_query.unwrap();

    let mut samples = vec![Some(TFr::default()); max_width]; // std::vec![TFr; max_width];

    for i in 0..10 {
        let known_ratio = 0.5 + (i as f32) * 0.05;
        let known: usize = ((max_width as f32) * known_ratio) as usize;

        for _ in 0..4 {
            random_missing(&mut samples, &data, max_width, known);

            let recovered_query = TPolyRecover::recover_poly_from_samples(&samples, &fs);
            assert!(recovered_query.is_ok());
            let recovered = recovered_query.unwrap();

            for (j, item) in data.iter().enumerate().take(max_width) {
                assert!(item.equals(&recovered.get_coeff_at(j)));
            }

            let mut recovered_vec: Vec<TFr> = Vec::new();

            for i in 0..max_width {
                recovered_vec.push(recovered.get_coeff_at(i));
            }

            let back_query = fs.fft_fr(&recovered_vec, true);
            assert!(back_query.is_ok());
            let back = back_query.unwrap();

            for (i, back_x) in back[..(max_width / 2)].iter().enumerate() {
                assert!(back_x.equals(&poly.get_coeff_at(i)));
            }

            for back_x in back[(max_width / 2)..max_width].iter() {
                assert!(back_x.is_zero());
            }
        }
    }
}
