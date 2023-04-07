use kzg::FFTFr;
use kzg::FFTSettings;
use kzg::Fr;
use kzg::Poly;
use kzg::PolyRecover;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::convert::TryInto;

pub fn recover_simple<
    TFr: Fr,
    TFTTSettings: FFTSettings<TFr> + FFTFr<TFr>,
    TPoly: Poly<TFr>,
    TPolyRecover: PolyRecover<TFr, TPoly, TFTTSettings>,
>() {
    let fs = TFTTSettings::new(2).unwrap();
    let max_width: usize = fs.get_max_width();

    let mut poly = vec![TFr::zero(); max_width];

    for (i, p) in poly.iter_mut().enumerate().take(max_width / 2) {
        *p = TFr::from_u64(i.try_into().unwrap());
    }

    //I think it is not needed, since vec! is set as Fr::zero(), but leaving just in case
    // for i in (max_width / 2)..max_width {
    //     poly[i] = TFr::zero();
    // }

    let data = fs.fft_fr(&poly, false).unwrap();
    let samples: [Option<TFr>; 4] = [Some(data[0].clone()), None, None, Some(data[3].clone())];

    let recovered = TPolyRecover::recover_poly_from_samples(&samples, &fs).unwrap();

    //Check recovered data
    assert_eq!(data.len(), max_width);
    for (i, d) in data.iter().enumerate() {
        assert!(d.equals(&recovered.get_coeff_at(i)));
    }

    let mut recovered_vec: Vec<TFr> = vec![];

    for i in 0..max_width {
        recovered_vec.push(recovered.get_coeff_at(i));
    }

    //Also check against original coefficients
    let back = fs.fft_fr(&recovered_vec, true).unwrap();
    for (i, p) in poly.iter().enumerate().take(max_width / 2) {
        assert!(p.equals(&back[i]));
    }

    for p in poly.iter().take(max_width).skip(max_width / 2) {
        assert!(p.is_zero());
    }
}

pub fn recover_random<
    TFr: Fr,
    TFTTSettings: FFTSettings<TFr> + FFTFr<TFr>,
    TPoly: Poly<TFr>,
    TPolyRecover: PolyRecover<TFr, TPoly, TFTTSettings>,
>() {
    let fs = TFTTSettings::new(12).unwrap();
    let max_width: usize = fs.get_max_width();

    let mut poly = vec![TFr::zero(); max_width];

    for (i, p) in poly.iter_mut().enumerate().take(max_width / 2) {
        *p = TFr::from_u64(i.try_into().unwrap());
    }

    let data = fs.fft_fr(&poly, false).unwrap();

    //Having half of the data is the minimum
    let mut known_ratio: f64 = 0.5;
    while known_ratio < 1.0 {
        let known: u64 = (max_width as f64 * known_ratio) as u64;
        for _ in 0..4 {
            let samples = random_missing(data.clone(), max_width, known);

            let recovered = TPolyRecover::recover_poly_from_samples(&samples, &fs).unwrap();
            //Assert
            assert_eq!(data.len(), max_width);
            for (i, d) in data.iter().enumerate() {
                assert!(d.equals(&recovered.get_coeff_at(i)));
            }

            let recovered_vec = (0..max_width)
                .map(|i| recovered.get_coeff_at(i))
                .collect::<Vec<_>>();

            //Also check against original coefficients
            let back = fs.fft_fr(&recovered_vec, true).unwrap();
            for i in 0..(max_width / 2) {
                assert!(poly[i].equals(&back[i]));
            }

            for p in poly.iter().take(max_width).skip(max_width / 2) {
                assert!(p.is_zero());
            }
        }

        //loop increment
        known_ratio += 0.05;
    }
}

pub fn more_than_half_missing<
    TFr: Fr,
    TFTTSettings: FFTSettings<TFr> + FFTFr<TFr>,
    TPoly: Poly<TFr>,
    TPolyRecover: PolyRecover<TFr, TPoly, TFTTSettings>,
>() {
    let fs = TFTTSettings::new(2).unwrap();
    let max_width: usize = fs.get_max_width();

    let mut poly = vec![TFr::zero(); max_width];

    for (i, p) in poly.iter_mut().enumerate().take(max_width / 2) {
        *p = TFr::from_u64(i.try_into().unwrap());
    }

    let data = fs.fft_fr(&poly, false).unwrap();
    let samples: [Option<TFr>; 4] = [Some(data[0].clone()), None, None, None];

    assert!(TPolyRecover::recover_poly_from_samples(&samples, &fs).is_err());
    assert!(TPolyRecover::recover_poly_from_samples(&[None], &fs).is_err());
}

fn random_missing<TFr: Fr>(data: Vec<TFr>, len_data: usize, known: u64) -> Vec<Option<TFr>> {
    let mut missing_idx: Vec<usize> = vec![];
    let mut with_missing = data.into_iter().map(Some).collect::<Vec<_>>();

    for i in 0..len_data {
        missing_idx.push(i);
    }

    missing_idx.shuffle(&mut thread_rng());

    for missing_idx in missing_idx.into_iter().take(len_data - (known as usize)) {
        with_missing[missing_idx] = None;
    }
    with_missing
}
