use mcl_rust::data_types::fr::Fr;
use mcl_rust::data_types::fp::Fp;
use mcl_rust::BlstFr;
use mcl_rust::BlstFp;
use mcl_rust::data_converter::fr_converter::*;
use mcl_rust::data_converter::fp_converter::*;
use mcl_rust::mcl_methods::init;
use mcl_rust::CurveType;

#[test]
fn convertBlstFrCorrectZero() {
    let blstZero = BlstFr::default();
    let converted = frFromBlst(blstZero);

    assert_eq!(converted, Fr::default());
}

#[test]
fn convertBlstFrCorrectRandomInts() {
    assert!(init(CurveType::BLS12_381));
    for i in 1..6 {
        let coef: i32 = i * (i * 3);
        let mut coefU64: u64 = 0;
        unsafe {
            coefU64 = coef as u64;
        }
        
        let mut blstFr: BlstFr = BlstFr::default();
        unsafe {
            blst::blst_fr_from_uint64(&mut blstFr, &coefU64);
        }
        let converted = frFromBlst(blstFr);

        assert_eq!(converted, Fr::from_int(coef));
    }
}

#[test]
fn convertToBlstFrCorrectZero() {
    let frZero = Fr::default();
    let converted = frToBlst(frZero);

    assert_eq!(converted, BlstFr::default());
}

#[test]
fn convertToBlstFrCorrectRandomInts() {
    assert!(init(CurveType::BLS12_381));
    for i in 1..6 {
        let coef: i32 = i * (i * 3);
        let mut coefU64: u64 = 0;
        unsafe {
            coefU64 = coef as u64;
        }

        let mut blstFr: BlstFr = BlstFr::default();
        unsafe {
            blst::blst_fr_from_uint64(&mut blstFr, &coefU64);
        }

        let converted = frToBlst(Fr::from_int(coef));

        assert_eq!(blstFr, converted);
    }
}

#[test]
fn convertBlstFpCorrectZero() {
    assert!(init(CurveType::BLS12_381));
    let blstZero = BlstFp::default();
    let converted = fpFromBlst(blstZero);

    assert_eq!(converted, Fp::default());
}