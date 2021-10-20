use mcl_rust::data_types::fr::Fr;
use mcl_rust::data_types::fp::Fp;
use mcl_rust::BlstFr;
use mcl_rust::BlstFp;
use mcl_rust::data_converter::fr_converter::*;
use mcl_rust::data_converter::fp_converter::*;
use mcl_rust::mcl_methods::init;
use mcl_rust::CurveType;

#[test]
fn convert_blst_fr_zero() {
    let blst_zero = BlstFr::default();
    let converted = fr_from_blst(blst_zero);

    assert_eq!(converted, Fr::default());
}

#[test]
fn convert_blst_fr_random_ints() {
    assert!(init(CurveType::BLS12_381));
    for i in 1..6 {
        let coef: i32 = i * (i * 3);
        let coef_u64: u64 = coef as u64;
        
        let mut blst_fr: BlstFr = BlstFr::default();
        unsafe {
            blst::blst_fr_from_uint64(&mut blst_fr, &coef_u64);
        }
        let converted = fr_from_blst(blst_fr);

        assert_eq!(converted, Fr::from_int(coef));
    }
}

#[test]
fn convert_to_blst_fr_zero() {
    let fr_zero = Fr::default();
    let converted = fr_to_blst(fr_zero);

    assert_eq!(converted, BlstFr::default());
}

#[test]
fn convert_to_blst_fr_random_ints() {
    assert!(init(CurveType::BLS12_381));
    for i in 1..6 {
        let coef: i32 = i * (i * 3);
        let coef_u64: u64 = coef as u64;
        
        let mut blst_fr: BlstFr = BlstFr::default();
        unsafe {
            blst::blst_fr_from_uint64(&mut blst_fr, &coef_u64);
        }

        let converted = fr_to_blst(Fr::from_int(coef));

        assert_eq!(blst_fr, converted);
    }
}

#[test]
fn convert_blst_fp_zero() {
    let blst_zero = BlstFp::default();
    let converted = fp_from_blst(blst_zero);

    assert_eq!(converted, Fp::default());
}