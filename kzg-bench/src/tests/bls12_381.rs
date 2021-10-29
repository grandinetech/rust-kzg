use kzg::Fr;
	
	
// pub fn log_2_byte_works () {
	// assert_eq!(0, log_2_byte(0x01));
	// assert_eq!(7, log_2_byte(0x80));
	// assert_eq!(7, log_2_byte(0xff));
	// assert_eq!(4, log_2_byte(0x10));
	
// }

pub fn fr_is_zero_works<TFr: Fr>() {
    let zero = TFr::from_u64(0);
	
	assert!(zero.is_zero());
}

pub fn fr_is_one_works<TFr: Fr>() {
	let one = TFr::from_u64(1);
	
	assert!(one.is_one());
}

// void fr_is_null_works(void) {
    // TEST_CHECK(fr_is_null(&fr_null));
    // TEST_CHECK(!fr_is_null(&fr_zero));
    // TEST_CHECK(!fr_is_null(&fr_one));
// }

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

// // pub fn fr_uint64s_roundtrip<TFr: Fr>() {
    // // fr_t fr;
    // // uint64_t expected[4] = {1, 2, 3, 4};
    // // uint64_t actual[4];
	
	
	
	
	// let mut fr = TFr::default();
	// let expected: [u64; 4] = {1, 2, 3, 4};
	
    // // fr_from_uint64s(&fr, expected);
    // // fr_to_uint64s(actual, &fr);

	// let fr = TFr::from_u64_arr(expected);
	// let actual = TFr::from_u64_arr(&fr);

    // // TEST_CHECK(expected[0] == actual[0]);
    // // TEST_CHECK(expected[1] == actual[1]);
    // // TEST_CHECK(expected[2] == actual[2]);
    // // TEST_CHECK(expected[3] == actual[3]);
// // }

// pub fn p1_mul_works<TFr: Fr, TG1: G1>() {
    // fr_t minus1;
    // g1_t res;
	
	// pub const G1_GENERATOR: TG1 = TG1 {
    // x: blst_fp { l: [0x5cb38790fd530c16, 0x7817fc679976fff5, 0x154f95c7143ba1c1, 0xf0ae6acdf3d0e747, 0xedce6ecc21dbf440, 0x120177419e0bfb75] },
    // y: blst_fp { l: [0xbaac93d50ce72271, 0x8c22631a7918fd8e, 0xdd595f13570725ce, 0x51ac582950405194, 0x0e1c8c3fad0059c0, 0x0bbc3efc5008a26a] },
    // z: blst_fp { l: [0x760900000002fffd, 0xebf4000bc40c0002, 0x5f48985753c758ba, 0x77ce585370525745, 0x5c071a97a256ec6d, 0x15f65ec3fa80e493] },
	// };
	
	// let m1: [u64; 4] = [0xffffffff00000000, 0x53bda402fffe5bfe, 0x3339d80809a1d805, 0x73eda753299d7d48];
	// let minus1 = TFr::from_u64_arr(&m1);
	
	
	
	// let res = TG1::mul(&G1_GENERATOR, &minus1);

	// assert!(TG1.equal

    // // Multiply the generator by minus one (the second root of unity)
    // fr_from_uint64s(&minus1, m1);
    // g1_mul(&res, &g1_generator, &minus1);

    // // We should end up with negative the generator
    // TEST_CHECK(g1_equal(&res, &g1_negative_generator));
// }

// void p1_sub_works(void) {
    // g1_t tmp, res;

    // // 2 * g1_gen = g1_gen - g1_gen_neg
    // g1_dbl(&tmp, &g1_generator);
    // g1_sub(&res, &g1_generator, &g1_negative_generator);

    // TEST_CHECK(g1_equal(&tmp, &res));
// }

// void p2_add_or_dbl_works(void) {
    // g2_t expected, actual;

    // g2_dbl(&expected, &g2_generator);
    // g2_add_or_dbl(&actual, &g2_generator, &g2_generator);

    // TEST_CHECK(g2_equal(&expected, &actual));
// }

// void p2_mul_works(void) {
    // fr_t minus1;
    // g2_t res;

    // // Multiply the generator by minus one (the second root of unity)
    // fr_from_uint64s(&minus1, m1);
    // g2_mul(&res, &g2_generator, &minus1);

    // TEST_CHECK(g2_equal(&res, &g2_negative_generator));
// }

// void p2_sub_works(void) {
    // g2_t tmp, res;

    // // 2 * g2_gen = g2_gen - g2_gen_neg
    // g2_dbl(&tmp, &g2_generator);
    // g2_sub(&res, &g2_generator, &g2_negative_generator);

    // TEST_CHECK(g2_equal(&tmp, &res));
// }

// void g1_identity_is_infinity(void) {
    // TEST_CHECK(g1_is_inf(&g1_identity));
// }

// void g1_identity_is_identity(void) {
    // g1_t actual;
    // g1_add_or_dbl(&actual, &g1_generator, &g1_identity);
    // TEST_CHECK(g1_equal(&g1_generator, &actual));
// }

// void g1_make_linear_combination(void) {
    // int len = 255;
    // fr_t coeffs[len], tmp;
    // g1_t p[len], res, exp;
    // for (int i = 0; i < len; i++) {
        // fr_from_uint64(coeffs + i, i + 1);
        // p[i] = g1_generator;
    // }

    // // Expected result
    // fr_from_uint64(&tmp, len * (len + 1) / 2);
    // g1_mul(&exp, &g1_generator, &tmp);

    // // Test result
    // g1_linear_combination(&res, p, coeffs, len);
    // TEST_CHECK(g1_equal(&exp, &res));
// }

// void g1_random_linear_combination(void) {
    // int len = 8192;
    // fr_t coeffs[len];
    // g1_t p[len], p1tmp = g1_generator;
    // for (int i = 0; i < len; i++) {
        // coeffs[i] = rand_fr();
        // p[i] = p1tmp;
        // g1_dbl(&p1tmp, &p1tmp);
    // }

    // // Expected result
    // g1_t exp = g1_identity;
    // for (uint64_t i = 0; i < len; i++) {
        // g1_mul(&p1tmp, &p[i], &coeffs[i]);
        // g1_add_or_dbl(&exp, &exp, &p1tmp);
    // }

    // // Test result
    // g1_t res;
    // g1_linear_combination(&res, p, coeffs, len);
    // TEST_CHECK(g1_equal(&exp, &res));
// }

// void pairings_work(void) {
    // // Verify that e([3]g1, [5]g2) = e([5]g1, [3]g2)
    // fr_t three, five;
    // g1_t g1_3, g1_5;
    // g2_t g2_3, g2_5;

    // // Set up
    // fr_from_uint64(&three, 3);
    // fr_from_uint64(&five, 5);
    // g1_mul(&g1_3, &g1_generator, &three);
    // g1_mul(&g1_5, &g1_generator, &five);
    // g2_mul(&g2_3, &g2_generator, &three);
    // g2_mul(&g2_5, &g2_generator, &five);

    // // Verify the pairing
    // TEST_CHECK(true == pairings_verify(&g1_3, &g2_5, &g1_5, &g2_3));
    // TEST_CHECK(false == pairings_verify(&g1_3, &g2_3, &g1_5, &g2_5));
// }