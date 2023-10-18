#[cfg(test)]
mod tests {
    use kzg_bench::tests::kzg_proofs::{
        commit_to_nil_poly, commit_to_too_long_poly_returns_err, proof_multi, proof_single,
    };
    use rust_kzg_arkworks::kzg_proofs::{generate_trusted_setup, FFTSettings, KZGSettings};
    use rust_kzg_arkworks::kzg_types::{ArkFr, ArkG1, ArkG2};
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
        commit_to_too_long_poly_returns_err::<
            ArkFr,
            ArkG1,
            ArkG2,
            PolyData,
            FFTSettings,
            KZGSettings,
        >(&generate_trusted_setup);
    }

    #[test]
    fn proof_multi_() {
        proof_multi::<ArkFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings>(
            &generate_trusted_setup,
        );
    }
}
