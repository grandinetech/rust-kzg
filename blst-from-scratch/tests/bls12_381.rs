#[cfg(test)]
pub mod tests {
    use kzg_bench::tests::bls12_381::{
        fr_div_by_zero, fr_div_works, fr_equal_works, fr_from_uint64_works, fr_is_null_works,
        fr_is_one_works, fr_is_zero_works, fr_negate_works, fr_pow_works, fr_uint64s_roundtrip,
        g1_identity_is_identity, g1_identity_is_infinity, g1_make_linear_combination,
        g1_random_linear_combination, log_2_byte_works, p1_mul_works, p1_sub_works,
        p2_add_or_dbl_works, p2_mul_works, p2_sub_works, pairings_work,
    };

    use blst_from_scratch::kzg_proofs::{g1_linear_combination, pairings_verify};
    use blst_from_scratch::types::fr::FsFr;
    use blst_from_scratch::types::g1::FsG1;
    use blst_from_scratch::types::g2::FsG2;
    use blst_from_scratch::utils::log_2_byte;

    #[test]
    fn log_2_byte_works_() {
        log_2_byte_works(&log_2_byte)
    }

    #[test]
    fn fr_is_null_works_() {
        fr_is_null_works::<FsFr>()
    }

    #[test]
    fn fr_is_zero_works_() {
        fr_is_zero_works::<FsFr>()
    }

    #[test]
    fn fr_is_one_works_() {
        fr_is_one_works::<FsFr>()
    }

    #[test]
    fn fr_from_uint64_works_() {
        fr_from_uint64_works::<FsFr>()
    }

    #[test]
    fn fr_equal_works_() {
        fr_equal_works::<FsFr>()
    }

    #[test]
    fn fr_negate_works_() {
        fr_negate_works::<FsFr>()
    }

    #[test]
    fn fr_pow_works_() {
        fr_pow_works::<FsFr>()
    }

    #[test]
    fn fr_div_works_() {
        fr_div_works::<FsFr>()
    }

    #[test]
    fn fr_div_by_zero_() {
        fr_div_by_zero::<FsFr>()
    }

    #[test]
    fn fr_uint64s_roundtrip_() {
        fr_uint64s_roundtrip::<FsFr>()
    }

    #[test]
    fn p1_mul_works_() {
        p1_mul_works::<FsFr, FsG1>()
    }

    #[test]
    fn p1_sub_works_() {
        p1_sub_works::<FsG1>()
    }

    #[test]
    fn p2_add_or_dbl_works_() {
        p2_add_or_dbl_works::<FsG2>()
    }

    #[test]
    fn p2_mul_works_() {
        p2_mul_works::<FsFr, FsG2>()
    }

    #[test]
    fn p2_sub_works_() {
        p2_sub_works::<FsG2>()
    }

    #[test]
    fn g1_identity_is_infinity_() {
        g1_identity_is_infinity::<FsG1>()
    }

    #[test]
    fn g1_identity_is_identity_() {
        g1_identity_is_identity::<FsG1>()
    }

    #[test]
    fn g1_make_linear_combination_() {
        g1_make_linear_combination::<FsFr, FsG1>(&g1_linear_combination)
    }

    #[test]
    fn g1_random_linear_combination_() {
        g1_random_linear_combination::<FsFr, FsG1>(&g1_linear_combination)
    }

    #[test]
    fn pairings_work_() {
        pairings_work::<FsFr, FsG1, FsG2>(&pairings_verify)
    }
}
