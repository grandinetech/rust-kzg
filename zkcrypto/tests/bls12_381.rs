#[cfg(test)]
mod tests {
    use kzg::common_utils::log_2_byte;
    use kzg_bench::tests::bls12_381::*;
    use rust_kzg_zkcrypto::fft_g1::g1_linear_combination;
    use rust_kzg_zkcrypto::kzg_proofs::pairings_verify;
    use rust_kzg_zkcrypto::kzg_types::{ZFp, ZFr, ZG1Affine, ZG1ProjAddAffine, ZG1, ZG2};

    #[test]
    pub fn log_2_byte_works_() {
        log_2_byte_works(&log_2_byte);
    }

    #[test]
    pub fn fr_is_zero_works_() {
        fr_is_zero_works::<ZFr>();
    }

    #[test]
    pub fn fr_is_one_works_() {
        fr_is_one_works::<ZFr>();
    }

    #[test]
    pub fn fr_from_uint64_works_() {
        fr_from_uint64_works::<ZFr>();
    }

    #[test]
    pub fn fr_equal_works_() {
        fr_equal_works::<ZFr>();
    }

    #[test]
    pub fn fr_negate_works_() {
        fr_negate_works::<ZFr>();
    }

    #[test]
    pub fn fr_pow_works_() {
        fr_pow_works::<ZFr>();
    }

    #[test]
    pub fn fr_div_works_() {
        fr_div_works::<ZFr>();
    }

    #[test]
    #[should_panic]
    pub fn fr_div_by_zero_() {
        fr_div_by_zero::<ZFr>();
    }

    #[test]
    pub fn fr_uint64s_roundtrip_() {
        fr_uint64s_roundtrip::<ZFr>();
    }

    #[test]
    pub fn p1_mul_works_() {
        p1_mul_works::<ZFr, ZG1>();
    }

    #[test]
    pub fn p1_sub_works_() {
        p1_sub_works::<ZG1>();
    }

    #[test]
    pub fn p2_add_or_dbl_works_() {
        p2_add_or_dbl_works::<ZG2>();
    }

    #[test]
    pub fn p2_mul_works_() {
        p2_mul_works::<ZFr, ZG2>();
    }

    #[test]
    pub fn p2_sub_works_() {
        p2_sub_works::<ZG2>();
    }

    #[test]
    pub fn g1_identity_is_infinity_() {
        g1_identity_is_infinity::<ZG1>();
    }

    #[test]
    pub fn g1_identity_is_identity_() {
        g1_identity_is_identity::<ZG1>();
    }

    #[test]
    pub fn g1_make_linear_combination_() {
        g1_make_linear_combination::<ZFr, ZG1, ZFp, ZG1Affine, ZG1ProjAddAffine>(
            &g1_linear_combination,
        );
    }

    #[test]
    pub fn g1_random_linear_combination_() {
        g1_random_linear_combination::<ZFr, ZG1, ZFp, ZG1Affine, ZG1ProjAddAffine>(
            &g1_linear_combination,
        );
    }

    #[test]
    pub fn pairings_work_() {
        pairings_work::<ZFr, ZG1, ZG2>(&pairings_verify);
    }

    #[test]
    pub fn fr_is_null_works_() {
        fr_is_null_works::<ZFr>();
    }
}
