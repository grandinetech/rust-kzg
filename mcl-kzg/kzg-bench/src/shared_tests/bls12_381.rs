#[cfg(test)]
pub mod tests {
    use kzg_bench::tests::bls12_381::*;
    use mcl_rust::data_types::fr::Fr;
    use mcl_rust::data_types::g1::mclBnG1_mulVec;
    use mcl_rust::data_types::g1::G1;
    use mcl_rust::data_types::g2::G2;
    use mcl_rust::kzg10::Curve;
    use mcl_rust::mcl_methods::init;
    use mcl_rust::utilities::log_2_byte;
    use mcl_rust::CurveType;

    #[test]
    pub fn log_2_byte_works_() {
        assert!(init(CurveType::BLS12_381));
        log_2_byte_works(&log_2_byte);
    }

    #[test]
    pub fn fr_is_null_works_() {
        assert!(init(CurveType::BLS12_381));
        fr_is_null_works::<Fr>();
    }

    #[test]
    pub fn fr_is_zero_works_() {
        assert!(init(CurveType::BLS12_381));
        fr_is_zero_works::<Fr>();
    }

    #[test]
    pub fn fr_is_one_works_() {
        assert!(init(CurveType::BLS12_381));
        fr_is_one_works::<Fr>();
    }

    #[test]
    pub fn fr_from_uint64_works_() {
        assert!(init(CurveType::BLS12_381));
        fr_from_uint64_works::<Fr>();
    }

    #[test]
    pub fn fr_equal_works_() {
        assert!(init(CurveType::BLS12_381));
        fr_equal_works::<Fr>();
    }

    #[test]
    pub fn fr_negate_works_() {
        assert!(init(CurveType::BLS12_381));
        fr_negate_works::<Fr>();
    }

    #[test]
    pub fn fr_pow_works_() {
        assert!(init(CurveType::BLS12_381));
        fr_pow_works::<Fr>();
    }

    #[test]
    pub fn fr_div_works_() {
        assert!(init(CurveType::BLS12_381));
        fr_div_works::<Fr>();
    }

    #[test]
    pub fn fr_div_by_zero_() {
        assert!(init(CurveType::BLS12_381));
        //fr_div_by_zero::<Fr>();
    }

    #[test]
    pub fn fr_uint64s_roundtrip_() {
        assert!(init(CurveType::BLS12_381));
        fr_uint64s_roundtrip::<Fr>();
    }

    #[test]
    pub fn p1_mul_works_() {
        assert!(init(CurveType::BLS12_381));
        p1_mul_works::<Fr, G1>();
    }

    #[test]
    pub fn p1_sub_works_() {
        assert!(init(CurveType::BLS12_381));
        p1_sub_works::<G1>();
    }

    #[test]
    pub fn p2_add_or_dbl_works_() {
        assert!(init(CurveType::BLS12_381));
        p2_add_or_dbl_works::<G2>();
    }

    #[test]
    pub fn p2_mul_works_() {
        assert!(init(CurveType::BLS12_381));
        p2_mul_works::<Fr, G2>();
    }

    #[test]
    pub fn p2_sub_works_() {
        assert!(init(CurveType::BLS12_381));
        p2_sub_works::<G2>();
    }

    #[test]
    pub fn g1_identity_is_infinity_() {
        assert!(init(CurveType::BLS12_381));
        g1_identity_is_infinity::<G1>();
    }

    #[test]
    pub fn g1_identity_is_identity_() {
        assert!(init(CurveType::BLS12_381));
        g1_identity_is_identity::<G1>();
    }

    #[test]
    pub fn g1_make_linear_combination_() {
        assert!(init(CurveType::BLS12_381));
        g1_make_linear_combination::<Fr, G1>(&g1_linear_combination);
    }

    #[test]
    pub fn g1_random_linear_combination_() {
        assert!(init(CurveType::BLS12_381));
        g1_random_linear_combination::<Fr, G1>(&g1_linear_combination);
    }

    #[test]
    pub fn pairings_work_() {
        assert!(init(CurveType::BLS12_381));
        pairings_work::<Fr, G1, G2>(&Curve::verify_pairing);
    }

    fn g1_linear_combination(result: &mut G1, g1_points: &[G1], coeffs: &[Fr], n: usize) {
        unsafe { mclBnG1_mulVec(result, g1_points.as_ptr(), coeffs.as_ptr(), n) }
    }
}
