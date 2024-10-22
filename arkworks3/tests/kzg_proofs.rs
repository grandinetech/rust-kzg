#[cfg(test)]
mod tests {
    use kzg_bench::tests::kzg_proofs::{
        commit_to_nil_poly, commit_to_too_long_poly_returns_err, proof_multi, proof_single,
    };
    use rust_kzg_arkworks3::kzg_proofs::{generate_trusted_setup, FFTSettings, KZGSettings};
    use rust_kzg_arkworks3::kzg_types::{ArkFp, ArkFr, ArkG1, ArkG1Affine, ArkG2};
    use rust_kzg_arkworks3::utils::PolyData;

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn proof_single_() {
        proof_single::<ArkFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings, ArkFp, ArkG1Affine>(
            &generate_trusted_setup,
        );
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn commit_to_nil_poly_() {
        commit_to_nil_poly::<
            ArkFr,
            ArkG1,
            ArkG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            ArkFp,
            ArkG1Affine,
        >(&generate_trusted_setup);
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn commit_to_too_long_poly_() {
        commit_to_too_long_poly_returns_err::<
            ArkFr,
            ArkG1,
            ArkG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            ArkFp,
            ArkG1Affine,
        >(&generate_trusted_setup);
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn proof_multi_() {
        proof_multi::<ArkFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings, ArkFp, ArkG1Affine>(
            &generate_trusted_setup,
        );
    }
}
