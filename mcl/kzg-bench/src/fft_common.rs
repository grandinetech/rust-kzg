use mcl_rust::data_types::fr::Fr;
use mcl_rust::fk20_fft::*;
use mcl_rust::mcl_methods::init;
use mcl_rust::CurveType;
use std::vec;

#[test]
fn fftsettings_new_creates_valid_settings() {
    // Arrange
    assert!(init(CurveType::BLS12_381));
    // Act
    let fft_settings = FFTSettings::new(4 + 5 + 1);
    // Assert
    let expected = vec![
        Fr::from_int(1),
        Fr::from_str(
            "21328829733576761151404230261968752855781179864716879432436835449516750606329",
            10,
        )
        .unwrap(),
        Fr::from_str(
            "12531186154666751577774347439625638674013361494693625348921624593362229945844",
            10,
        )
        .unwrap(),
        Fr::from_str(
            "36815421669481109810171413925233110915304823983913164224028689762034127238951",
            10,
        )
        .unwrap(),
    ];

    // has lots more members, but checking the first few should point out whether the math is correct
    let starting_equal = fft_settings
        .expanded_roots_of_unity
        .iter()
        .zip(expected)
        .all(|(a, b)| a.get_str(10) == b.get_str(10));

    assert!(starting_equal);
}
