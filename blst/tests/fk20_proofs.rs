#[cfg(test)]
mod tests {
    use kzg_bench::tests::fk20_proofs::*;
    use rust_kzg_blst::types::fft_settings::FsFFTSettings;
    use rust_kzg_blst::types::fk20_multi_settings::FsFK20MultiSettings;
    use rust_kzg_blst::types::fk20_single_settings::FsFK20SingleSettings;
    use rust_kzg_blst::types::fp::FsFp;
    use rust_kzg_blst::types::fr::FsFr;
    use rust_kzg_blst::types::g1::{FsG1, FsG1Affine};
    use rust_kzg_blst::types::g2::FsG2;
    use rust_kzg_blst::types::kzg_settings::FsKZGSettings;
    use rust_kzg_blst::types::poly::FsPoly;
    use rust_kzg_blst::utils::generate_trusted_setup;

    #[test]
    fn test_fk_single() {
        fk_single::<
            FsFr,
            FsG1,
            FsG2,
            FsPoly,
            FsFFTSettings,
            FsKZGSettings,
            FsFK20SingleSettings,
            FsFp,
            FsG1Affine,
        >(&generate_trusted_setup);
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
            FsFp,
            FsG1Affine,
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
            FsFp,
            FsG1Affine,
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
            FsFp,
            FsG1Affine,
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
            FsFp,
            FsG1Affine,
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
            FsFp,
            FsG1Affine,
        >(&generate_trusted_setup);
    }
}
