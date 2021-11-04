#[cfg(test)]
pub mod tests {
    use kzg_bench::tests::kzg_proofs::{proof_single, commit_to_nil_poly, commit_to_too_long_poly, proof_multi};
    use arkworks::kzg_proofs::{FFTSettings, KZGSettings, generate_trusted_setup};
    use arkworks::kzg_types::{FsFr, ArkG1, ArkG2};
    use arkworks::utils::PolyData;

    #[test]
    fn proof_single_() {
        proof_single::<FsFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings>(&generate_trusted_setup);
    }
    #[test]
    fn commit_to_nil_poly_() {
        commit_to_nil_poly::<FsFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings>(&generate_trusted_setup);
    }
    #[test]
    #[should_panic(expected = "Poly given is too long")]
    fn commit_to_too_long_poly_() {
        commit_to_too_long_poly::<FsFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings>(&generate_trusted_setup);
    }

    // #[test]
    fn proof_multi_() {
        proof_multi::<FsFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings>(&generate_trusted_setup);
    }
}