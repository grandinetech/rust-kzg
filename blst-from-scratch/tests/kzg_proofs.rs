#[cfg(test)]
mod tests {
    use kzg_bench::tests::kzg_proofs::{commit_to_nil_poly, commit_to_too_long_poly_returns_err, proof_multi, proof_single};

    use blst_from_scratch::types::fft_settings::FsFFTSettings;
    use blst_from_scratch::types::fr::FsFr;
    use blst_from_scratch::types::g1::FsG1;
    use blst_from_scratch::types::g2::FsG2;
    use blst_from_scratch::types::kzg_settings::FsKZGSettings;
    use blst_from_scratch::types::poly::FsPoly;
    use blst_from_scratch::utils::generate_trusted_setup;

    #[test]
    pub fn test_proof_single() {
        proof_single::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &generate_trusted_setup,
        );
    }

    #[test]
    pub fn test_commit_to_nil_poly() {
        commit_to_nil_poly::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &generate_trusted_setup,
        );
    }

    #[test]
    pub fn test_commit_to_too_long_poly() {
        commit_to_too_long_poly_returns_err::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &generate_trusted_setup,
        );
    }

    #[test]
    pub fn test_proof_multi() {
        proof_multi::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
            &generate_trusted_setup,
        );
    }
}
