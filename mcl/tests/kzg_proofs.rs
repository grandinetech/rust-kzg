#[cfg(test)]
mod tests {
    use kzg::G1;
    use kzg_bench::tests::kzg_proofs::{
        commit_to_nil_poly, commit_to_too_long_poly_returns_err, proof_multi, proof_single,
    };

    use rust_kzg_mcl::types::fft_settings::FsFFTSettings;
    use rust_kzg_mcl::types::fp::FsFp;
    use rust_kzg_mcl::types::fr::FsFr;
    use rust_kzg_mcl::types::g1::{FsG1, FsG1Affine};
    use rust_kzg_mcl::types::g2::FsG2;
    use rust_kzg_mcl::types::kzg_settings::FsKZGSettings;
    use rust_kzg_mcl::types::poly::FsPoly;
    use rust_kzg_mcl::utils::generate_trusted_setup;

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    pub fn test_proof_single() {
        proof_single::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings, FsFp, FsG1Affine>(
            &generate_trusted_setup,
        );
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    pub fn test_commit_to_nil_poly() {
        commit_to_nil_poly::<
            FsFr,
            FsG1,
            FsG2,
            FsPoly,
            FsFFTSettings,
            FsKZGSettings,
            FsFp,
            FsG1Affine,
        >(&generate_trusted_setup);
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    pub fn test_commit_to_too_long_poly() {
        commit_to_too_long_poly_returns_err::<
            FsFr,
            FsG1,
            FsG2,
            FsPoly,
            FsFFTSettings,
            FsKZGSettings,
            FsFp,
            FsG1Affine,
        >(&generate_trusted_setup);
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    pub fn test_proof_multi() {
        proof_multi::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings, FsFp, FsG1Affine>(
            &generate_trusted_setup,
        );
    }
}
