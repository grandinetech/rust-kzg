#[cfg(test)]
mod tests {
    use kzg_bench::tests::bls12_381::*;
    use ckzg::consts::{BlstP1, BlstP2};
    use ckzg::finite::BlstFr;
    use ckzg::utils::log_2_byte;

    #[test]
    fn test_log_2_byte_works() {
        log_2_byte_works(&log_2_byte);
    }

    #[test]
    fn test_fr_is_zero_works() {
        fr_is_zero_works::<BlstFr>();
    }

    #[test]
    fn test_fr_is_one_works() {
        fr_is_one_works::<BlstFr>();
    }

    #[test]
    fn test_fr_from_uint64_works() {
        fr_from_uint64_works::<BlstFr>();
    }

    #[test]
    fn test_fr_equal_works() {
        fr_equal_works::<BlstFr>();
    }

    #[test]
    fn test_fr_negate_works() {
        fr_negate_works::<BlstFr>();
    }

    #[test]
    fn test_fr_pow_works() {
        //fr_pow_works::<BlstFr>();
    }

    #[test]
    fn test_fr_div_works() {
        fr_div_works::<BlstFr>();
    }

    #[test]
    fn test_fr_div_by_zero() {
        fr_div_by_zero::<BlstFr>();
    }

    #[test]
    fn test_fr_uint64s_roundtrip() {
        //fr_uint64s_roundtrip::<BlstFr>();
    }

    #[test]
    fn test_p1_mul_works() {
        p1_mul_works::<BlstFr, BlstP1>();
    }

    #[test]
    fn test_p1_sub_works() {
        p1_sub_works::<BlstP1>();
    }

    #[test]
    fn test_p2_add_or_dbl_works() {
        p2_add_or_dbl_works::<BlstP2>();
    }

    #[test]
    fn test_p2_mul_works() {
        p2_mul_works::<BlstFr, BlstP2>();
    }

    #[test]
    fn test_p2_sub_works() {
        p2_sub_works::<BlstP2>();
    }

    #[test]
    fn test_g1_identity_is_infinity() {
        g1_identity_is_infinity::<BlstP1>();
    }

    #[test]
    fn test_g1_identity_is_identity() {
        g1_identity_is_identity::<BlstP1>();
    }
}
