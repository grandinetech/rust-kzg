#[cfg(test)]
mod tests {
    use kzg_bench::tests::fk20_proofs::*;

    use rust_kzg_zkcrypto::fk20_proofs::{KzgFK20MultiSettings, KzgFK20SingleSettings};
    use rust_kzg_zkcrypto::kzg_proofs::{generate_trusted_setup, FFTSettings, KZGSettings};
    use rust_kzg_zkcrypto::kzg_types::{ZFp, ZFr as BlstFr, ZG1Affine};
    use rust_kzg_zkcrypto::kzg_types::{ZG1, ZG2};
    use rust_kzg_zkcrypto::poly::PolyData;

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn test_fk_single() {
        fk_single::<
            BlstFr,
            ZG1,
            ZG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            KzgFK20SingleSettings,
            ZFp,
            ZG1Affine,
        >(&generate_trusted_setup);
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn test_fk_single_strided() {
        fk_single_strided::<
            BlstFr,
            ZG1,
            ZG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            KzgFK20SingleSettings,
            ZFp,
            ZG1Affine,
        >(&generate_trusted_setup);
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn test_fk_multi_settings() {
        fk_multi_settings::<
            BlstFr,
            ZG1,
            ZG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            KzgFK20MultiSettings,
            ZFp,
            ZG1Affine,
        >(&generate_trusted_setup);
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn test_fk_multi_chunk_len_1_512() {
        fk_multi_chunk_len_1_512::<
            BlstFr,
            ZG1,
            ZG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            KzgFK20MultiSettings,
            ZFp,
            ZG1Affine,
        >(&generate_trusted_setup);
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn test_fk_multi_chunk_len_16_512() {
        fk_multi_chunk_len_16_512::<
            BlstFr,
            ZG1,
            ZG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            KzgFK20MultiSettings,
            ZFp,
            ZG1Affine,
        >(&generate_trusted_setup);
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn test_fk_multi_chunk_len_16_16() {
        fk_multi_chunk_len_16_16::<
            BlstFr,
            ZG1,
            ZG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            KzgFK20MultiSettings,
            ZFp,
            ZG1Affine,
        >(&generate_trusted_setup);
    }
}
