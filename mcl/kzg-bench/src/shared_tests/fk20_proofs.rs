#[cfg(test)]
mod tests {
    use kzg_bench::tests::fk20_proofs::*;
    use rust_kzg_mcl::data_types::{fr::Fr, g1::G1, g2::G2};
    use rust_kzg_mcl::fk20_fft::FFTSettings;
    use rust_kzg_mcl::fk20_matrix::{FK20Matrix, FK20SingleMatrix};
    use rust_kzg_mcl::kzg10::Polynomial;
    use rust_kzg_mcl::kzg_settings::KZGSettings;
    use rust_kzg_mcl::mcl_methods::init;
    use rust_kzg_mcl::CurveType;

    #[test]
    fn test_fk_single() {
        assert!(init(CurveType::BLS12_381));
        fk_single::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings, FK20SingleMatrix>(
            &KZGSettings::generate_trusted_setup,
        );
    }

    #[test]
    fn test_fk_single_strided() {
        assert!(init(CurveType::BLS12_381));
        fk_single_strided::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings, FK20SingleMatrix>(
            &KZGSettings::generate_trusted_setup,
        );
    }

    #[test]
    fn test_fk_multi_settings() {
        assert!(init(CurveType::BLS12_381));
        fk_multi_settings::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings, FK20Matrix>(
            &KZGSettings::generate_trusted_setup,
        );
    }

    #[test]
    fn test_fk_multi_chunk_len_1_512() {
        assert!(init(CurveType::BLS12_381));
        fk_multi_chunk_len_1_512::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings, FK20Matrix>(
            &KZGSettings::generate_trusted_setup,
        );
    }

    #[test]
    fn test_fk_multi_chunk_len_16_512() {
        assert!(init(CurveType::BLS12_381));
        fk_multi_chunk_len_16_512::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings, FK20Matrix>(
            &KZGSettings::generate_trusted_setup,
        );
    }

    #[test]
    fn test_fk_multi_chunk_len_16_16() {
        assert!(init(CurveType::BLS12_381));
        fk_multi_chunk_len_16_16::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings, FK20Matrix>(
            &KZGSettings::generate_trusted_setup,
        );
    }
}
