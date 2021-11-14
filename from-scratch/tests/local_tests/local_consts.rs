use kzg::{FFTSettings, Fr};

pub fn roots_of_unity_repeat_at_stride<TFr: Fr, TFFTSettings: FFTSettings<TFr>>() {
    let fs1 = TFFTSettings::new(15).unwrap();
    let fs2 = TFFTSettings::new(16).unwrap();
    let fs3 = TFFTSettings::new(17).unwrap();

    for i in 0..fs1.get_max_width() {
        assert!(fs1.get_expanded_roots_of_unity_at(i).equals(&fs2.get_expanded_roots_of_unity_at(i * 2)));
        assert!(fs1.get_expanded_roots_of_unity_at(i).equals(&fs3.get_expanded_roots_of_unity_at(i * 4)));
    }
}