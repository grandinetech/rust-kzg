#[cfg(test)]
mod tests {
    use blst_rust::types::fft_settings::FsFFTSettings;
    use blst_rust::types::fk20_multi_settings::FsFK20MultiSettings;
    use blst_rust::types::fk20_single_settings::FsFK20SingleSettings;
    use blst_rust::types::fr::FsFr;
    use blst_rust::types::g1::FsG1;
    use blst_rust::types::g2::FsG2;
    use blst_rust::types::kzg_settings::FsKZGSettings;
    use blst_rust::types::poly::FsPoly;
    use blst_rust::utils::generate_trusted_setup;
    use kzg_bench::tests::fk20_proofs::*;

    #[test]
    fn test_fk_single() {
        fk_single::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings, FsFK20SingleSettings>(
            &generate_trusted_setup,
        );
    }

    #[test]
    fn test_fk_single_strided() {
        fk_single_strided::<
            FsFr,
            FsG1,
            FsG2,
            FsPoly,
            FsFFTSettings,
            FsKZGSettings,
            FsFK20SingleSettings,
        >(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_settings() {
        fk_multi_settings::<
            FsFr,
            FsG1,
            FsG2,
            FsPoly,
            FsFFTSettings,
            FsKZGSettings,
            FsFK20MultiSettings,
        >(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_1_512() {
        fk_multi_chunk_len_1_512::<
            FsFr,
            FsG1,
            FsG2,
            FsPoly,
            FsFFTSettings,
            FsKZGSettings,
            FsFK20MultiSettings,
        >(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_16_512() {
        fk_multi_chunk_len_16_512::<
            FsFr,
            FsG1,
            FsG2,
            FsPoly,
            FsFFTSettings,
            FsKZGSettings,
            FsFK20MultiSettings,
        >(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_16_16() {
        fk_multi_chunk_len_16_16::<
            FsFr,
            FsG1,
            FsG2,
            FsPoly,
            FsFFTSettings,
            FsKZGSettings,
            FsFK20MultiSettings,
        >(&generate_trusted_setup);
    }
}
