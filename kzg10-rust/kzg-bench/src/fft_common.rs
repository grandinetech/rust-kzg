use std::{mem, vec};
use mcl_rust::CurveType;
use mcl_rust::data_types::fr::Fr;
use mcl_rust::mcl_methods::init;
use mcl_rust::fk20_fft::*;

#[test]
fn fftsettings_new_creates_valid_settings() {
    // Arrange
    assert!(init(CurveType::BLS12_381));
    // Act
    let fft_settings = FFTSettings::new(4 + 5 + 1);
    
    // Assert
    let expected = vec![
        Fr::from_int(1),
        Fr::from_str("21328829733576761151404230261968752855781179864716879432436835449516750606329", 10).unwrap(),
        Fr::from_str("12531186154666751577774347439625638674013361494693625348921624593362229945844", 10).unwrap(),
        Fr::from_str("36815421669481109810171413925233110915304823983913164224028689762034127238951", 10).unwrap()
    ];

    // has lots more members, but checking the first few should point out whether the math is correct
    let starting_equal =fft_settings.exp_roots_of_unity.iter()
        .zip(expected)
        .all(|(a, b)| a.get_str(10) == b.get_str(10));

    assert!(starting_equal);
}

#[test]
fn root_of_unity_is_the_expected_size() {
    //Arrange
    let element_size: usize;
    let length: usize;
    unsafe{
        element_size = mem::size_of_val(&SCALE_2_ROOT_OF_UNITY[0]);
        length = SCALE_2_ROOT_OF_UNITY.len()
    }
    
    //Act/Assert
    assert_eq!(mem::size_of::<Fr>(), element_size);
    assert_eq!(32, length);
}

#[test]
/*Library could be improved to use status codes/Result<T, E> or something like that,
since in current implementation it simply panics*/
// #[should_panic]
fn roots_of_unity_out_of_bounds_fails() {
    //Arrange
    assert!(init(CurveType::BLS12_381));

    //Act
    let settings = std::panic::catch_unwind(|| FFTSettings::new(32));
    
    //Assert
    assert!(settings.is_err());
}

#[test]
fn roots_of_unity_are_plausible() {
    //Arrange
    assert!(init(CurveType::BLS12_381));
    let mut i = 0;
    let mut r: Fr;

    //Act/Assert
    while i < 32 {
        unsafe{
            r = SCALE_2_ROOT_OF_UNITY[i].clone();
        }
        
        let mut j = 0;
        while j < i {
            let cl = r.clone();
            Fr::sqr(&mut r, &cl);
            j = j + 1;
        }
        assert!(r.is_one());
        i = i + 1;
    }
}

#[test]
fn expand_roots_is_plausible() {
    //Arrange
    assert!(init(CurveType::BLS12_381));
    const SCALE: usize = 15;
    const WIDTH: usize = 1 << SCALE;
    let root: Fr;
    let mut prod: Fr = Fr::zero();

    //Act/Assert
    unsafe{
        init_globals();
        root = SCALE_2_ROOT_OF_UNITY[SCALE].clone();
    }
    let expanded = expand_root_of_unity(&root);
    assert!(expanded[0].is_one());
    assert!(expanded[WIDTH].is_one());

    for i in 1..(WIDTH/2 + 1) {
        Fr::mul(&mut prod, &expanded[i], &expanded[WIDTH - i]);
        assert!(prod.is_one());
    }

}