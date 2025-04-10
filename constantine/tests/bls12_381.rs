#[cfg(test)]
mod tests {
    use kzg::common_utils::log_2_byte;
    use kzg_bench::tests::bls12_381::{
        fr_div_by_zero, fr_div_works, fr_equal_works, fr_from_uint64_works, fr_is_null_works,
        fr_is_one_works, fr_is_zero_works, fr_negate_works, fr_pow_works, fr_uint64s_roundtrip,
        g1_identity_is_identity, g1_identity_is_infinity, g1_make_linear_combination,
        g1_random_linear_combination, g1_small_linear_combination, log_2_byte_works, p1_mul_works,
        p1_sub_works, p2_add_or_dbl_works, p2_mul_works, p2_sub_works, pairings_work,
    };

    use rust_kzg_constantine::kzg_proofs::{g1_linear_combination, pairings_verify};
    use rust_kzg_constantine::types::fp::CtFp;
    use rust_kzg_constantine::types::fr::CtFr;
    use rust_kzg_constantine::types::g1::{CtG1, CtG1Affine, CtG1ProjAddAffine};
    use rust_kzg_constantine::types::g2::CtG2;

    #[test]
    fn log_2_byte_works_() {
        log_2_byte_works(&log_2_byte)
    }

    #[test]
    fn fr_is_null_works_() {
        fr_is_null_works::<CtFr>()
    }

    #[test]
    fn fr_is_zero_works_() {
        fr_is_zero_works::<CtFr>()
    }

    #[test]
    fn fr_is_one_works_() {
        fr_is_one_works::<CtFr>()
    }

    #[test]
    fn fr_from_uint64_works_() {
        fr_from_uint64_works::<CtFr>()
    }

    #[test]
    fn fr_equal_works_() {
        fr_equal_works::<CtFr>()
    }

    #[test]
    fn fr_negate_works_() {
        fr_negate_works::<CtFr>()
    }

    #[test]
    fn fr_pow_works_() {
        fr_pow_works::<CtFr>()
    }

    #[test]
    fn fr_div_works_() {
        fr_div_works::<CtFr>()
    }

    #[test]
    fn fr_div_by_zero_() {
        fr_div_by_zero::<CtFr>()
    }

    #[test]
    fn fr_uint64s_roundtrip_() {
        fr_uint64s_roundtrip::<CtFr>()
    }

    #[test]
    fn p1_mul_works_() {
        p1_mul_works::<CtFr, CtG1>()
    }

    #[test]
    fn p1_sub_works_() {
        p1_sub_works::<CtG1>()
    }

    #[test]
    fn p2_add_or_dbl_works_() {
        p2_add_or_dbl_works::<CtG2>()
    }

    #[test]
    fn p2_mul_works_() {
        p2_mul_works::<CtFr, CtG2>()
    }

    #[test]
    fn p2_sub_works_() {
        p2_sub_works::<CtG2>()
    }

    #[test]
    fn g1_identity_is_infinity_() {
        g1_identity_is_infinity::<CtG1>()
    }

    #[test]
    fn g1_identity_is_identity_() {
        g1_identity_is_identity::<CtG1>()
    }

    #[test]
    fn g1_make_linear_combination_() {
        g1_make_linear_combination::<CtFr, CtG1, CtFp, CtG1Affine, CtG1ProjAddAffine>(
            &g1_linear_combination,
        )
    }

    #[test]
    fn g1_random_linear_combination_() {
        g1_random_linear_combination::<CtFr, CtG1, CtFp, CtG1Affine, CtG1ProjAddAffine>(
            &g1_linear_combination,
        )
    }

    #[test]
    fn g1_small_linear_combination_() {
        g1_small_linear_combination::<CtFr, CtG1, CtFp, CtG1Affine, CtG1ProjAddAffine>(
            &g1_linear_combination,
        )
    }

    #[test]
    fn pairings_work_() {
        pairings_work::<CtFr, CtG1, CtG2>(&pairings_verify)
    }
}
