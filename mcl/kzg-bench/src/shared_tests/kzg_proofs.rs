#[cfg(test)]
mod kzg_proofs_tests {
    use kzg_bench::tests::kzg_proofs::*;
    use mcl_rust::data_types::{fr::Fr, g1::G1, g2::G2};
    use mcl_rust::fk20_fft::FFTSettings;
    use mcl_rust::kzg10::Polynomial;
    use mcl_rust::kzg_settings::KZGSettings;
    use mcl_rust::mcl_methods::init;
    use mcl_rust::CurveType;

    #[test]
    fn proof_single_() {
        assert!(init(CurveType::BLS12_381));
        proof_single::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings>(
            &KZGSettings::generate_trusted_setup,
        );
    }

    #[test]
    fn commit_to_nil_poly_() {
        assert!(init(CurveType::BLS12_381));
        commit_to_nil_poly::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings>(
            &KZGSettings::generate_trusted_setup,
        );
    }

    #[test]
    fn commit_to_too_long_poly_returns_err_() {
        assert!(init(CurveType::BLS12_381));
        commit_to_too_long_poly_returns_err::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings>(
            &KZGSettings::generate_trusted_setup,
        );
    }

    #[test]
    fn proof_multi_() {
        assert!(init(CurveType::BLS12_381));
        proof_multi::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings>(
            &KZGSettings::generate_trusted_setup,
        );
    }
}
