#[cfg(test)]
mod tests {
    use kzg_bench::tests::fk20_proofs::*;
    use zkcrypto::fftsettings::ZkFFTSettings;
    use zkcrypto::fk20::{ZkFK20MultiSettings, ZkFK20SingleSettings};
    use zkcrypto::kzg_proofs::{generate_trusted_setup, KZGSettings};
    use zkcrypto::kzg_types::{ZkG1Projective, ZkG2Projective};
    use zkcrypto::poly::ZPoly;
    use zkcrypto::zkfr::blsScalar;

    #[test]
    fn test_fk_single() {
        fk_single::<
            blsScalar,
            ZkG1Projective,
            ZkG2Projective,
            ZPoly,
            ZkFFTSettings,
            KZGSettings,
            ZkFK20SingleSettings,
        >(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_single_strided() {
        fk_single_strided::<
            blsScalar,
            ZkG1Projective,
            ZkG2Projective,
            ZPoly,
            ZkFFTSettings,
            KZGSettings,
            ZkFK20SingleSettings,
        >(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_settings() {
        fk_multi_settings::<
            blsScalar,
            ZkG1Projective,
            ZkG2Projective,
            ZPoly,
            ZkFFTSettings,
            KZGSettings,
            ZkFK20MultiSettings,
        >(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_1_512() {
        fk_multi_chunk_len_1_512::<
            blsScalar,
            ZkG1Projective,
            ZkG2Projective,
            ZPoly,
            ZkFFTSettings,
            KZGSettings,
            ZkFK20MultiSettings,
        >(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_16_512() {
        fk_multi_chunk_len_16_512::<
            blsScalar,
            ZkG1Projective,
            ZkG2Projective,
            ZPoly,
            ZkFFTSettings,
            KZGSettings,
            ZkFK20MultiSettings,
        >(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_16_16() {
        fk_multi_chunk_len_16_16::<
            blsScalar,
            ZkG1Projective,
            ZkG2Projective,
            ZPoly,
            ZkFFTSettings,
            KZGSettings,
            ZkFK20MultiSettings,
        >(&generate_trusted_setup);
    }
}
