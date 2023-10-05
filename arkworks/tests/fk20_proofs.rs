#[cfg(test)]
mod tests {
    use kzg_bench::tests::fk20_proofs::*;

    use rust_kzg_arkworks::fk20_proofs::{KzgFK20MultiSettings, KzgFK20SingleSettings};
    use rust_kzg_arkworks::kzg_proofs::{generate_trusted_setup, FFTSettings, KZGSettings};
    use rust_kzg_arkworks::kzg_types::ArkFr as BlstFr;
    use rust_kzg_arkworks::kzg_types::{ArkG1, ArkG2};
    use rust_kzg_arkworks::utils::PolyData;

    #[test]
    fn test_fk_single() {
        fk_single::<BlstFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings, KzgFK20SingleSettings>(
            &generate_trusted_setup,
        );
    }

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
        >(&generate_trusted_setup);
    }

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
        >(&generate_trusted_setup);
    }

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
        >(&generate_trusted_setup);
    }

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
        >(&generate_trusted_setup);
    }

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
        >(&generate_trusted_setup);
    }
}
