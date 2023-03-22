#[cfg(test)]
mod tests {
    use arkworks::kzg_proofs::{generate_trusted_setup, FFTSettings, KZGSettings};
    use arkworks::kzg_types::{ArkG1, ArkG2, FsFr};
    use arkworks::utils::PolyData;
    use kzg_bench::tests::kzg_proofs::{
        commit_to_nil_poly, commit_to_too_long_poly, proof_multi, proof_single,
    };

    #[test]
    fn proof_single_() {
        proof_single::<FsFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings>(
            &generate_trusted_setup,
        );
    }
    #[test]
    fn commit_to_nil_poly_() {
        commit_to_nil_poly::<FsFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings>(
            &generate_trusted_setup,
        );
    }
    #[test]
    #[should_panic(expected = "Poly given is too long")]
    fn commit_to_too_long_poly_() {
        commit_to_too_long_poly::<FsFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings>(
            &generate_trusted_setup,
        );
    }

    #[test]
    fn proof_multi_() {
        proof_multi::<FsFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings>(
            &generate_trusted_setup,
        );
    }
}
