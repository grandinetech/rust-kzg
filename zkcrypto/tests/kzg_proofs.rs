#[cfg(test)]
mod tests {
    use kzg_bench::tests::kzg_proofs::{
        commit_to_nil_poly, commit_to_too_long_poly_returns_err, proof_multi, proof_single,
    };
    use rust_kzg_zkcrypto::kzg_proofs::{generate_trusted_setup, FFTSettings, KZGSettings};
    use rust_kzg_zkcrypto::kzg_types::{ZFp, ZFr, ZG1Affine, ZG1, ZG2};
    use rust_kzg_zkcrypto::poly::PolyData;

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn proof_single_() {
        proof_single::<ZFr, ZG1, ZG2, PolyData, FFTSettings, KZGSettings, ZFp, ZG1Affine>(
            &generate_trusted_setup,
        );
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn commit_to_nil_poly_() {
        commit_to_nil_poly::<ZFr, ZG1, ZG2, PolyData, FFTSettings, KZGSettings, ZFp, ZG1Affine>(
            &generate_trusted_setup,
        );
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn commit_to_too_long_poly_() {
        commit_to_too_long_poly_returns_err::<
            ZFr,
            ZG1,
            ZG2,
            PolyData,
            FFTSettings,
            KZGSettings,
            ZFp,
            ZG1Affine,
        >(&generate_trusted_setup);
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    fn proof_multi_() {
        proof_multi::<ZFr, ZG1, ZG2, PolyData, FFTSettings, KZGSettings, ZFp, ZG1Affine>(
            &generate_trusted_setup,
        );
    }
}
