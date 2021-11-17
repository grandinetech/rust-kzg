use kzg::FFTSettings;
use kzg::Fr;
use kzg::Poly;
use kzg::PolyRecover;
use kzg::FFTFr;
use std::convert::TryInto;
use rand::thread_rng;
use rand::seq::SliceRandom;

pub fn recover_simple<
    TFr: Fr,
    TFTTSettings: FFTSettings<TFr> + FFTFr<TFr>,
    TPoly: Poly<TFr>,
    TPolyRecover: PolyRecover<TFr, TPoly, TFTTSettings>
>() {
    let fs = TFTTSettings::new(2).unwrap();
    let max_width: usize = fs.get_max_width();

    let mut poly = vec![TFr::zero(); max_width];

    for i in 0..(max_width / 2) {
        poly[i] = TFr::from_u64(i.try_into().unwrap());
    }

    //I think it is not needed, since vec! is set as Fr::zero(), but leaving just in case
    // for i in (max_width / 2)..max_width {
    //     poly[i] = TFr::zero();
    // }
    
    let data = fs.fft_fr(&poly, false).unwrap();
    let samples: [Option<TFr>; 4] = [Some(data[0].clone()), None, None, Some(data[3].clone())];

    let recovered = TPolyRecover::recover_poly_from_samples(&samples, &fs);

    //Check recovered data
    for i in 0..max_width {
        assert!(data[i].equals(&recovered.get_coeff_at(i)));
    }

    let mut recovered_vec: Vec<TFr> = vec![];

    for i in 0..max_width {
        recovered_vec.push(recovered.get_coeff_at(i));
    }

    //Also check against original coefficients
    let back = fs.fft_fr(&recovered_vec, true).unwrap();
    for i in 0..(max_width / 2) {
        assert!(poly[i].equals(&back[i]));
    }

    for i in (max_width / 2)..max_width {
        assert!(poly[i].is_zero());
    }
}

pub fn recover_random<
    TFr: Fr,
    TFTTSettings: FFTSettings<TFr> + FFTFr<TFr>,
    TPoly: Poly<TFr>,
    TPolyRecover: PolyRecover<TFr, TPoly, TFTTSettings>
>() {

    let fs = TFTTSettings::new(12).unwrap();
    let max_width: usize = fs.get_max_width();

    let mut poly = vec![TFr::zero(); max_width];

    for i in 0..(max_width / 2) {
        poly[i] = TFr::from_u64(i.try_into().unwrap());
    }

    let data = fs.fft_fr(&poly, false).unwrap();

    //Having half of the data is the minimum
    let mut known_ratio: f64 = 0.5;
    while known_ratio < 1.0 {
        let known: u64 = (max_width as f64 * known_ratio) as u64;
        for _ in 0..4 {
            let samples = random_missing(data.clone(), max_width, known);


            let recovered = TPolyRecover::recover_poly_from_samples(&samples, &fs);
            //Assert
            for i in 0..max_width {
                assert!(data[i].equals(&recovered.get_coeff_at(i)));
            }
        
            let mut recovered_vec: Vec<TFr> = vec![];
            for i in 0..max_width {
                recovered_vec.push(recovered.get_coeff_at(i));
            }
        
            //Also check against original coefficients
            let back = fs.fft_fr(&recovered_vec, true).unwrap();
            for i in 0..(max_width / 2) {
                assert!(poly[i].equals(&back[i]));
            }
        
            for i in (max_width / 2)..max_width {
                assert!(poly[i].is_zero());
            }
        }

        //loop increment
        known_ratio += 0.05;
    }
}

fn random_missing<TFr: Fr>(data: Vec<TFr>, len_data: usize, known: u64) -> Vec<Option<TFr>> {
    let mut missin_idx: Vec<usize> = vec![];
    let mut with_missing: Vec<Option<TFr>> = vec![];

    for i in 0..len_data {
        missin_idx.push(i);
    }

    missin_idx.shuffle(&mut thread_rng());

    for i in 0..len_data {
        with_missing.push(Some(data[i].clone()));
    }

    for i in 0..(len_data - (known as usize)) {
        with_missing[missin_idx[i]] = None;
    }
    println!("random_missing {} - {}", len_data, known);
    with_missing
}