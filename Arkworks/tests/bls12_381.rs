#[cfg(test)]
pub mod tests {
	use kzg_bench::tests::bls12_381::*;
    use arkworks::kzg_types::FsFr;
	use arkworks::kzg_types::ArkG1;
	use arkworks::kzg_types::ArkG2;

	// use arkworks::kzg_types::{G1_IDENTITY, G1_GENERATOR, G1_NEGATIVE_GENERATOR, G2_GENERATOR, G2_NEGATIVE_GENERATOR, pairings_verify, g1_linear_combination};
	// use zkcrypto::curve::fp::Fp as ZkFp;
	// use kzg::Fr;
	use arkworks::fft_g1::log_2_byte;
	use arkworks::utils::*;
    use arkworks::kzg_proofs::pairings_verify;
	
#[test]	
pub fn log_2_byte_works_() {
	log_2_byte_works(&log_2_byte);
	
}	
	
#[test]
pub fn fr_is_zero_works_() {
	fr_is_zero_works::<FsFr>();
}

#[test]
pub fn fr_is_one_works_() {
	fr_is_one_works::<FsFr>();
	
}

// pub fn fr_is_null_works(void) {
    // TEST_CHECK(fr_is_null(&fr_null));
    // TEST_CHECK(!fr_is_null(&fr_zero));
    // TEST_CHECK(!fr_is_null(&fr_one));
// }

#[test]
pub fn fr_from_uint64_works_() {
	fr_from_uint64_works::<FsFr>();
}

#[test]
pub fn fr_equal_works_() {
	fr_equal_works::<FsFr>();
}

#[test]
pub fn fr_negate_works_() {
    fr_negate_works::<FsFr>();
}

#[test]
pub fn fr_pow_works_() {
	fr_pow_works::<FsFr>();
}

#[test]
pub fn fr_div_works_() {
	fr_div_works::<FsFr>();
}

#[test]
#[should_panic]
pub fn fr_div_by_zero_() {
	fr_div_by_zero::<FsFr>();
}

#[test]
pub fn fr_uint64s_roundtrip_() {
	fr_uint64s_roundtrip::<FsFr>();	
}

#[test]
pub fn p1_mul_works_() {
	p1_mul_works::<FsFr, ArkG1>();
}

#[test]
pub fn p1_sub_works_() {
	p1_sub_works::<ArkG1>();
}

#[test]
pub fn p2_add_or_dbl_works_() {
	p2_add_or_dbl_works::<ArkG2>();
}

#[test]
pub fn p2_mul_works_() {
	p2_mul_works::<FsFr, ArkG2>();

}

#[test]
pub fn p2_sub_works_() {
	p2_sub_works::<ArkG2>();
}

#[test]
pub fn g1_identity_is_infinity_() {
	g1_identity_is_infinity::<ArkG1>();
}

#[test]
pub fn g1_identity_is_identity_() {
	g1_identity_is_identity::<ArkG1>();

}

// #[test]
// pub fn g1_make_linear_combination_() {
// 	g1_make_linear_combination::<FsFr, ArkG1>(G1_GENERATOR, &g1_linear_combination);

// }


#[test]
pub fn pairings_work_() {
	pairings_work::<FsFr, ArkG1, ArkG2>(&pairings_verify);
}

}