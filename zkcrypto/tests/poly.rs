#[cfg(test)]
pub mod tests {
    use kzg_bench::tests::poly::*;
    use zkcrypto::poly::{ZPoly, Fr};
	use zkcrypto::zkfr::blsScalar;
	use zkcrypto::utils::*;
	
    #[test]
	fn create_poly_of_length_ten_() {
		create_poly_of_length_ten::<blsScalar, ZPoly>();
	}
	
	#[test]
	fn poly_eval_check_() {
		poly_eval_check::<blsScalar, ZPoly>();
	}

	#[test]
	fn poly_eval_0_check_() {
		poly_eval_0_check::<blsScalar, ZPoly>();
	}

	#[test]
	fn poly_eval_nil_check_() {
		poly_eval_nil_check::<blsScalar, ZPoly>();
	}

	
	// not working for some reason
	#[test]
	fn poly_inverse_simple_0_() {
		poly_inverse_simple_0::<blsScalar, ZPoly>();
	}

	#[test]
	fn poly_inverse_simple_1_() {
		poly_inverse_simple_1::<blsScalar, ZPoly>();
	}
	
	#[test]
	fn conversion_one() {
		let mut first = blst::blst_fr::default();
		unsafe {
			blst::blst_fr_from_uint64(&mut first, [1, 0, 0, 0].as_ptr());
		}
		let second = blst_fr_into_zk_fr(&first);
		
		assert_eq!(second, <blsScalar as Fr>::one());
	}
	
	#[test]
	fn conversion_two() {
		let mut first = <blsScalar as Fr>::zero();
		let mut ret = blst::blst_fr::default();
		unsafe {
			blst::blst_fr_from_uint64(&mut ret, [0, 0, 0, 0].as_ptr());
		}
		let second = zk_fr_into_blst_fr(&first);
		assert_eq!(second, ret);
		
	}
}