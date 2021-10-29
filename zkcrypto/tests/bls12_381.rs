#[cfg(test)]
pub mod tests {
	use kzg_bench::tests::bls12_381::*;
    use zkcrypto::zkfr::blsScalar;
	use kzg::Fr;
	
#[test]
pub fn fr_is_zero_works_() {
	fr_is_zero_works::<blsScalar>();
}

#[test]
pub fn fr_is_one_works_() {
	fr_is_one_works::<blsScalar>();
	
}

// void fr_is_null_works(void) {
    // TEST_CHECK(fr_is_null(&fr_null));
    // TEST_CHECK(!fr_is_null(&fr_zero));
    // TEST_CHECK(!fr_is_null(&fr_one));
// }

#[test]
pub fn fr_from_uint64_works_() {
	fr_from_uint64_works::<blsScalar>();
}

#[test]
pub fn fr_equal_works_() {
	fr_equal_works::<blsScalar>();
}

#[test]
pub fn fr_negate_works_() {
    fr_negate_works::<blsScalar>();
}

#[test]
pub fn fr_pow_works_() {
	fr_pow_works::<blsScalar>();
}

#[test]
pub fn fr_div_works_() {
	fr_div_works::<blsScalar>();
}

#[test]
pub fn fr_div_by_zero_() {
	fr_div_by_zero::<blsScalar>();
}

// #[test]
// pub fn fr_uint64s_roundtrip() {
    // // fr_t fr;
    // // uint64_t expected[4] = {1, 2, 3, 4};
    // // uint64_t actual[4];
	
	// //let mut fr = blsScalar::default();
	// let expected: [u64; 4] = [1, 2, 3, 4];
	
    // // fr_from_uint64s(&fr, expected);
    // // fr_to_uint64s(actual, &fr);

	// let fr = blsScalar::from_u64_arr(&expected);
	// let mut ret: [u8; 32] = blsScalar::to_bytes(&fr);
	
	// let actual = u64::from(ret);
	
	// assert_eq!(expected[0], actual[0]);

    // // TEST_CHECK(expected[0] == actual[0]);
    // // TEST_CHECK(expected[1] == actual[1]);
    // // TEST_CHECK(expected[2] == actual[2]);
    // // TEST_CHECK(expected[3] == actual[3]);
// }


}