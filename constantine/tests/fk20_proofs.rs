#[cfg(test)]
mod tests {
    use kzg_bench::tests::fk20_proofs::*;
    use rust_kzg_constantine::eip_7594::CtBackend;
    use rust_kzg_constantine::types::fk20_multi_settings::CtFK20MultiSettings;
    use rust_kzg_constantine::types::fk20_single_settings::CtFK20SingleSettings;
    use rust_kzg_constantine::utils::generate_trusted_setup;

    #[test]
    fn test_fk_single() {
        fk_single::<CtBackend, CtFK20SingleSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_single_strided() {
        fk_single_strided::<CtBackend, CtFK20SingleSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_settings() {
        fk_multi_settings::<CtBackend, CtFK20MultiSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_1_512() {
        fk_multi_chunk_len_1_512::<CtBackend, CtFK20MultiSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_16_512() {
        fk_multi_chunk_len_16_512::<CtBackend, CtFK20MultiSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_16_16() {
        fk_multi_chunk_len_16_16::<CtBackend, CtFK20MultiSettings>(&generate_trusted_setup);
    }
}
