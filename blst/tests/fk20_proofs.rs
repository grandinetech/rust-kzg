#[cfg(test)]
mod tests {
    use kzg_bench::tests::fk20_proofs::*;
    use rust_kzg_blst::eip_7594::BlstBackend;
    use rust_kzg_blst::types::fk20_multi_settings::FsFK20MultiSettings;
    use rust_kzg_blst::types::fk20_single_settings::FsFK20SingleSettings;
    use rust_kzg_blst::utils::generate_trusted_setup;

    #[test]
    fn test_fk_single() {
        fk_single::<BlstBackend, FsFK20SingleSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_single_strided() {
        fk_single_strided::<BlstBackend, FsFK20SingleSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_settings() {
        fk_multi_settings::<BlstBackend, FsFK20MultiSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_1_512() {
        fk_multi_chunk_len_1_512::<BlstBackend, FsFK20MultiSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_16_512() {
        fk_multi_chunk_len_16_512::<BlstBackend, FsFK20MultiSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_16_16() {
        fk_multi_chunk_len_16_16::<BlstBackend, FsFK20MultiSettings>(&generate_trusted_setup);
    }
}
