use kzg::{Fr, G1_, G2_};	
use std::convert::TryInto;

pub fn log_2_byte_works (log_2_byte: &dyn Fn(u8) -> usize) {
	assert_eq!(0, log_2_byte(0x01));
	assert_eq!(7, log_2_byte(0x80));
	assert_eq!(7, log_2_byte(0xff));
	assert_eq!(4, log_2_byte(0x10));
}

pub fn fr_is_zero_works<TFr: Fr>() {
    let zero = TFr::from_u64(0);
	
	assert!(zero.is_zero());
}

pub fn fr_is_one_works<TFr: Fr>() {
	let one = TFr::from_u64(1);
	
	assert!(one.is_one());
}

pub fn fr_from_uint64_works<TFr: Fr>() {
    let a = TFr::from_u64(1);
	
	assert!(a.is_one());
}

pub fn fr_equal_works<TFr: Fr>() {
    // // A couple of arbitrary roots of unity
    let aa: [u64; 4] = [
	0x0001000000000000,
	0xec03000276030000,
	0x8d51ccce760304d0,
	0x0000000000000000
	];
	
	let bb: [u64; 4] = [
	0x8dd702cb688bc087,	
	0xa032824078eaa4fe,
	0xa733b23a98ca5b22,
	0x3f96405d25a31660
	];
	
	let a: TFr = TFr::from_u64_arr(&aa);
	let b: TFr = TFr::from_u64_arr(&bb);
	
	assert_eq!(true, a.equals(&a));
	assert_eq!(false, a.equals(&b));
}

pub fn fr_negate_works<TFr: Fr>() {
	let m1: [u64; 4] = [0xffffffff00000000, 0x53bda402fffe5bfe, 0x3339d80809a1d805, 0x73eda753299d7d48];

	let minus1 = TFr::from_u64_arr(&m1);
	
	let res = TFr::negate(&minus1);
	assert!(res.is_one());
}

pub fn fr_pow_works<TFr: Fr>() {
    // // a^pow
	let pow: u64 = 123456;
	let a = TFr::from_u64(197);
	
    // // Do it the slow way
    let expected = TFr::one();
	for _ in 0..pow {
		expected.mul(&a);
	}
	
    // // Do it the quick way
	let actual = a.pow(pow as usize);
	
	assert!(expected.equals(&actual));
}

pub fn fr_div_works<TFr: Fr>() {

	let a = TFr::from_u64(197);
	let b = TFr::from_u64(123456);
	
	let tmp = a.div(&b).unwrap();
	let actual = tmp.mul(&b);
	
	assert!(a.equals(&actual));
}

// // This is strictly undefined, but conventionally 0 is returned
pub fn fr_div_by_zero<TFr: Fr>() {	
	let a = TFr::from_u64(197);
	let b = TFr::from_u64(0);

	let tmp = a.div(&b).unwrap();
	
	assert!(tmp.is_zero());
}

pub fn fr_uint64s_roundtrip<TFr: Fr>() {
	let expected: [u64; 4] = [1, 2, 3, 4];
	
	let fr = TFr::from_u64_arr(&expected);
	let actual = TFr::to_u64_arr(&fr);

	assert_eq!(expected[0], actual[0]);
	assert_eq!(expected[1], actual[1]);
	assert_eq!(expected[2], actual[2]);
	assert_eq!(expected[3], actual[3]);	
}

pub fn p1_mul_works<TFr: Fr, TG1: G1_<TFr>>(g1_generator: &TG1, g1_negative_generator: &TG1) {
	let m1: [u64; 4] = [0xffffffff00000000, 0x53bda402fffe5bfe, 0x3339d80809a1d805, 0x73eda753299d7d48];
	let minus1 = TFr::from_u64_arr(&m1);	
	let res = TG1::mul(&g1_generator, &minus1);

	assert!(res.equals(&g1_negative_generator));	
}

pub fn p1_sub_works<TFr: Fr, TG1: G1_<TFr>>(g1_generator: &TG1, g1_negative_generator: &TG1) {
	
	let tmp = TG1::dbl(&g1_generator);
	let res = TG1::sub(&g1_generator, &g1_negative_generator);
	
	assert!(tmp.equals(&res));	
}

pub fn p2_add_or_dbl_works<TFr: Fr, TG2: G2_<TFr>>(g2_generator: &TG2) {
	let expected = TG2::dbl(&g2_generator);
	let actual = TG2::add_or_double(&g2_generator, &g2_generator);
	
	assert!(expected.equals(&actual));
}

pub fn p2_mul_works<TFr: Fr, TG2: G2_<TFr>>(g2_generator: &TG2, g2_negative_generator: &TG2) {
	let m1: [u64; 4] = [0xffffffff00000000, 0x53bda402fffe5bfe, 0x3339d80809a1d805, 0x73eda753299d7d48];

	let minus1 = TFr::from_u64_arr(&m1);
	let res = TG2::mul(&g2_generator, &minus1);
	
	assert!(res.equals(&g2_negative_generator));
}

pub fn p2_sub_works<TFr: Fr, TG2: G2_<TFr>>(g2_generator: &TG2, g2_negative_generator: &TG2) {
	
	let tmp = TG2::dbl(&g2_generator);
	let res = TG2::sub(&g2_generator, &g2_negative_generator);
	
	assert!(tmp.equals(&res));
}

pub fn g1_identity_is_infinity<TFr: Fr, TG1: G1_<TFr>>(g1_identity: &TG1) {

	assert!(TG1::is_inf(&g1_identity));
}

pub fn g1_identity_is_identity<TFr: Fr, TG1: G1_<TFr>>(g1_identity: &TG1, g1_generator: &TG1) {
	let actual = TG1::add_or_double(&g1_generator, &g1_identity);
	
	assert!(actual.equals(&g1_generator));
}

pub fn g1_make_linear_combination<TFr: Fr, TG1: G1_<TFr> + Copy>(g1_generator: TG1, g1_linear_combination: &dyn Fn (&mut TG1, &Vec<TG1>, &Vec<TFr>, usize)) {

	let len: usize = 255;
	let mut coeffs = vec![TFr::default(); len];
	
	let mut p = vec![TG1::default(); len];
	
	
	for i in 0..len {
		coeffs[i] = TFr::from_u64((i+1).try_into().unwrap());
		p[i] = g1_generator;
	}
	
	let tmp = TFr::from_u64((len * (len + 1) / 2).try_into().unwrap());
	let exp = TG1::mul(&g1_generator, &tmp);
	
	let mut res = TG1::default();
	
	g1_linear_combination(&mut res, &p, &coeffs, len);
	
	assert!(exp.equals(&res));
}

pub fn g1_random_linear_combination<TFr: Fr, TG1: G1_<TFr> + Copy>(g1_generator: TG1, g1_identity: &TG1, g1_linear_combination: &dyn Fn (&mut TG1, &Vec<TG1>, &Vec<TFr>, usize)) {
	let len: usize = 8192;
	let mut coeffs = vec![TFr::default(); len];
	let mut p = vec![TG1::default(); len];
	let mut p1tmp = g1_generator;
	
	for i in 0..len {
		coeffs[i] = TFr::rand();
		p[i] = p1tmp;
		p1tmp.dbl();
	}
	
	let exp = g1_identity;
	for i in 0..len {
		p1tmp = TG1::mul(&p[i], &coeffs[i]);
		exp.add_or_double(&p1tmp);	
	}
	
	let mut res = TG1::default();
	g1_linear_combination(&mut res, &p, &coeffs, len);
	
	assert!(exp.equals(&res));
}

pub fn pairings_work<TFr: Fr, TG1: G1_<TFr>, TG2: G2_<TFr>>(g1_generator: &TG1, g2_generator: &TG2, pairings_verify: &dyn Fn(&TG1, &TG2, &TG1, &TG2) -> bool) {
    // // Verify that e([3]g1, [5]g2) = e([5]g1, [3]g2)
	
	let three = TFr::from_u64(3);
	let five = TFr::from_u64(5);
	
	let g1_3 = TG1::mul(&g1_generator, &three);
	let g1_5 = TG1::mul(&g1_generator, &five);
	
	let g2_3 = TG2::mul(&g2_generator, &three);
	let g2_5 = TG2::mul(&g2_generator, &five);	
	
	assert_eq!(true, pairings_verify(&g1_3, &g2_5, &g1_5, &g2_3));
	assert_eq!(false, pairings_verify(&g1_3, &g2_3, &g1_5, &g2_5));
	
}