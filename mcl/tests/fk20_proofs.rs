#[cfg(test)]
mod tests {
    use kzg_bench::tests::fk20_proofs::*;
    use rust_kzg_mcl::eip_7594::MclBackend;
    use rust_kzg_mcl::types::fk20_multi_settings::FsFK20MultiSettings;
    use rust_kzg_mcl::types::fk20_single_settings::FsFK20SingleSettings;
    use rust_kzg_mcl::utils::generate_trusted_setup;

    #[test]
    fn test_fk_single() {
        fk_single::<MclBackend, FsFK20SingleSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_single_strided() {
        fk_single_strided::<MclBackend, FsFK20SingleSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_settings() {
        fk_multi_settings::<MclBackend, FsFK20MultiSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_1_512() {
        fk_multi_chunk_len_1_512::<MclBackend, FsFK20MultiSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_16_512() {
        fk_multi_chunk_len_16_512::<MclBackend, FsFK20MultiSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_16_16() {
        fk_multi_chunk_len_16_16::<MclBackend, FsFK20MultiSettings>(&generate_trusted_setup);
    }
}
