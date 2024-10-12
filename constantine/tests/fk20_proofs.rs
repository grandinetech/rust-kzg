#[cfg(test)]
mod tests {
    use kzg_bench::tests::fk20_proofs::*;
    use rust_kzg_constantine::types::fft_settings::CtFFTSettings;
    use rust_kzg_constantine::types::fk20_multi_settings::CtFK20MultiSettings;
    use rust_kzg_constantine::types::fk20_single_settings::CtFK20SingleSettings;
    use rust_kzg_constantine::types::fp::CtFp;
    use rust_kzg_constantine::types::fr::CtFr;
    use rust_kzg_constantine::types::g1::{CtG1, CtG1Affine};
    use rust_kzg_constantine::types::g2::CtG2;
    use rust_kzg_constantine::types::kzg_settings::CtKZGSettings;
    use rust_kzg_constantine::types::poly::CtPoly;
    use rust_kzg_constantine::utils::generate_trusted_setup;

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn test_fk_single() {
        fk_single::<
            CtFr,
            CtG1,
            CtG2,
            CtPoly,
            CtFFTSettings,
            CtKZGSettings,
            CtFK20SingleSettings,
            CtFp,
            CtG1Affine,
        >(&generate_trusted_setup);
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn test_fk_single_strided() {
        fk_single_strided::<
            CtFr,
            CtG1,
            CtG2,
            CtPoly,
            CtFFTSettings,
            CtKZGSettings,
            CtFK20SingleSettings,
            CtFp,
            CtG1Affine,
        >(&generate_trusted_setup);
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn test_fk_multi_settings() {
        fk_multi_settings::<
            CtFr,
            CtG1,
            CtG2,
            CtPoly,
            CtFFTSettings,
            CtKZGSettings,
            CtFK20MultiSettings,
            CtFp,
            CtG1Affine,
        >(&generate_trusted_setup);
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn test_fk_multi_chunk_len_1_512() {
        fk_multi_chunk_len_1_512::<
            CtFr,
            CtG1,
            CtG2,
            CtPoly,
            CtFFTSettings,
            CtKZGSettings,
            CtFK20MultiSettings,
            CtFp,
            CtG1Affine,
        >(&generate_trusted_setup);
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn test_fk_multi_chunk_len_16_512() {
        fk_multi_chunk_len_16_512::<
            CtFr,
            CtG1,
            CtG2,
            CtPoly,
            CtFFTSettings,
            CtKZGSettings,
            CtFK20MultiSettings,
            CtFp,
            CtG1Affine,
        >(&generate_trusted_setup);
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn test_fk_multi_chunk_len_16_16() {
        fk_multi_chunk_len_16_16::<
            CtFr,
            CtG1,
            CtG2,
            CtPoly,
            CtFFTSettings,
            CtKZGSettings,
            CtFK20MultiSettings,
            CtFp,
            CtG1Affine,
        >(&generate_trusted_setup);
    }
}
