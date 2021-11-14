use kzg::FFTSettings;
use kzg::Fr;
use kzg::Poly;
use kzg::PolyRecover;
use kzg::FFTFr;
use std::convert::TryInto;

pub fn recover_simple<TFr: Fr, TFTTSettings: FFTSettings<TFr> + FFTFr<TFr>, TPoly: Poly<TFr>, TPolyRecover: PolyRecover<TFr, TPoly, TFTTSettings>>() {
    let fs = TFTTSettings::new(2).unwrap();
    let max_width: usize = fs.get_max_width();

    let mut poly = vec![TFr::zero(); max_width];

    for i in 0..(max_width / 2) {
        poly[i] = TFr::from_u64(i.try_into().unwrap());
    }

    // for i in max_width / 2..max_width {
    //     poly[i] = TFr::zero();
    // }
    
    let data = fs.fft_fr(&poly, false).unwrap();
    let sample: [Option<TFr>; 4] = [Some(data[0].clone()), None, None, Some(data[3].clone())];

    let recovered = TPolyRecover::recover_poly_from_samples(&sample, fs);

    //Check recovered data
    for i in 0..max_width {
        assert!(poly[i].equals(&recovered.get_coeff_at(i)));
    }
}