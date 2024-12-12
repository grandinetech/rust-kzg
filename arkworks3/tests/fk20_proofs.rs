#[cfg(test)]
mod tests {
    use kzg_bench::tests::fk20_proofs::*;

    use rust_kzg_arkworks3::fk20_proofs::{KzgFK20MultiSettings, KzgFK20SingleSettings};
    use rust_kzg_arkworks3::kzg_proofs::{
        generate_trusted_setup, LFFTSettings as FFTSettings, LKZGSettings as KZGSettings,
    };
    use rust_kzg_arkworks3::kzg_types::{ArkFp, ArkFr as BlstFr, ArkG1Affine};
    use rust_kzg_arkworks3::kzg_types::{ArkG1, ArkG2};
    use rust_kzg_arkworks3::utils::PolyData;

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn test_fk_single() {
        fk_single::<
            BlstFr,
            ArkG1,
            ArkG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            KzgFK20SingleSettings,
            ArkFp,
            ArkG1Affine,
        >(&generate_trusted_setup);
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn test_fk_single_strided() {
        fk_single_strided::<
            BlstFr,
            ArkG1,
            ArkG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            KzgFK20SingleSettings,
            ArkFp,
            ArkG1Affine,
        >(&generate_trusted_setup);
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn test_fk_multi_settings() {
        fk_multi_settings::<
            BlstFr,
            ArkG1,
            ArkG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            KzgFK20MultiSettings,
            ArkFp,
            ArkG1Affine,
        >(&generate_trusted_setup);
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn test_fk_multi_chunk_len_1_512() {
        fk_multi_chunk_len_1_512::<
            BlstFr,
            ArkG1,
            ArkG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            KzgFK20MultiSettings,
            ArkFp,
            ArkG1Affine,
        >(&generate_trusted_setup);
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn test_fk_multi_chunk_len_16_512() {
        fk_multi_chunk_len_16_512::<
            BlstFr,
            ArkG1,
            ArkG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            KzgFK20MultiSettings,
            ArkFp,
            ArkG1Affine,
        >(&generate_trusted_setup);
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn test_fk_multi_chunk_len_16_16() {
        fk_multi_chunk_len_16_16::<
            BlstFr,
            ArkG1,
            ArkG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            KzgFK20MultiSettings,
            ArkFp,
            ArkG1Affine,
        >(&generate_trusted_setup);
    }
}
