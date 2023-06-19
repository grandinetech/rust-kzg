#[cfg(test)]
mod kzg_proofs_tests {
    use kzg_bench::tests::kzg_proofs::*;
    use rust_kzg_mcl::data_types::{fr::Fr, g1::G1, g2::G2};
    use rust_kzg_mcl::fk20_fft::FFTSettings;
    use rust_kzg_mcl::kzg10::Polynomial;
    use rust_kzg_mcl::kzg_settings::KZGSettings;
    use rust_kzg_mcl::mcl_methods::init;
    use rust_kzg_mcl::CurveType;

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
