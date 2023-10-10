#[cfg(test)]
mod tests {
    use kzg_bench::tests::kzg_proofs::{
        commit_to_nil_poly, proof_multi, proof_single, commit_to_too_long_poly_returns_err,
    };
    use rust_kzg_arkworks::kzg_proofs::{generate_trusted_setup, FFTSettings, KZGSettings};
    use rust_kzg_arkworks::kzg_types::{ArkG1, ArkG2, ArkFr};
    use rust_kzg_arkworks::utils::PolyData;

    #[test]
    fn proof_single_() {
        proof_single::<ArkFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings>(
            &generate_trusted_setup,
        );
    }
    #[test]
    fn commit_to_nil_poly_() {
        commit_to_nil_poly::<ArkFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings>(
            &generate_trusted_setup,
        );
    }
    #[test]
    fn commit_to_too_long_poly_() {
        commit_to_too_long_poly_returns_err::<ArkFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings>(
            &generate_trusted_setup,
        );
    }

    #[test]
    fn proof_multi_() {
        proof_multi::<ArkFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings>(
            &generate_trusted_setup,
        );
    }
}
