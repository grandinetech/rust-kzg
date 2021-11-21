#[cfg(test)]
mod tests {
    use blst_from_scratch::kzg_types::{FsKZGSettings, FsFFTSettings, FsPoly, FsFr, FsG1, FsG2};
    use blst_from_scratch::utils::{generate_trusted_setup};
    use kzg_bench::tests::kzg_proofs::{commit_to_nil_poly, commit_to_too_long_poly, proof_multi, proof_single};

    #[test]
    pub fn test_proof_single() {
        proof_single::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(&generate_trusted_setup);
    }

    #[test]
    pub fn test_commit_to_nil_poly() {
        commit_to_nil_poly::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(&generate_trusted_setup);
    }

    #[test]
    pub fn test_commit_to_too_long_poly() {
        commit_to_too_long_poly::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(&generate_trusted_setup);
    }

    #[test]
    pub fn test_proof_multi() {
        proof_multi::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(&generate_trusted_setup);
    }
}
