#[cfg(test)]
mod tests {
    use kzg_bench::tests::fk20_proofs::*;

    use rust_kzg_zkcrypto::eip_7594::ZBackend;
    use rust_kzg_zkcrypto::fk20_proofs::{KzgFK20MultiSettings, KzgFK20SingleSettings};
    use rust_kzg_zkcrypto::kzg_proofs::generate_trusted_setup;

    #[test]
    fn test_fk_single() {
        fk_single::<ZBackend, KzgFK20SingleSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_single_strided() {
        fk_single_strided::<ZBackend, KzgFK20SingleSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_settings() {
        fk_multi_settings::<ZBackend, KzgFK20MultiSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_1_512() {
        fk_multi_chunk_len_1_512::<ZBackend, KzgFK20MultiSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_16_512() {
        fk_multi_chunk_len_16_512::<ZBackend, KzgFK20MultiSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_16_16() {
        fk_multi_chunk_len_16_16::<ZBackend, KzgFK20MultiSettings>(&generate_trusted_setup);
    }
}
