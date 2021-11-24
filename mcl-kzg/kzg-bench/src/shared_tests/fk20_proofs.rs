#[cfg(test)]
mod tests {
    use kzg_bench::tests::fk20_proofs::*;
    use mcl_rust::data_types::{fr::Fr, g1::G1, g2::G2};
    use mcl_rust::fk20_fft::{FFTSettings};
    use mcl_rust::mcl_methods::init;
    use mcl_rust::CurveType;
    use mcl_rust::kzg_settings::KZGSettings;
    use mcl_rust::kzg10::Polynomial;
    use mcl_rust::fk20_matrix::{FK20Matrix, FK20SingleMatrix};

    #[test]
    fn test_fk_single() {
        assert!(init(CurveType::BLS12_381));
        fk_single::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings, FK20SingleMatrix>(&KZGSettings::generate_trusted_setup);
    }

    #[test]
    fn test_fk_single_strided() {
        assert!(init(CurveType::BLS12_381));
        fk_single_strided::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings, FK20SingleMatrix>(&KZGSettings::generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_settings() {
        assert!(init(CurveType::BLS12_381));
        fk_multi_settings::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings, FK20Matrix>(&KZGSettings::generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_1_512() {
        assert!(init(CurveType::BLS12_381));
        fk_multi_chunk_len_1_512::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings, FK20Matrix>(&KZGSettings::generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_16_512() {
        assert!(init(CurveType::BLS12_381));
        fk_multi_chunk_len_16_512::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings, FK20Matrix>(&KZGSettings::generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_16_16() {
        assert!(init(CurveType::BLS12_381));
        fk_multi_chunk_len_16_16::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings, FK20Matrix>(&KZGSettings::generate_trusted_setup);
    }
}