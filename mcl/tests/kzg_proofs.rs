#[cfg(test)]
mod tests {
    use kzg_bench::tests::kzg_proofs::{
        commit_to_nil_poly, commit_to_too_long_poly_returns_err, proof_multi, proof_single,
    };

    use rust_kzg_mcl::eip_7594::MclBackend;
    use rust_kzg_mcl::utils::generate_trusted_setup;

    #[test]
    pub fn test_proof_single() {
        proof_single::<MclBackend>(&generate_trusted_setup);
    }

    #[test]
    pub fn test_commit_to_nil_poly() {
        commit_to_nil_poly::<MclBackend>(&generate_trusted_setup);
    }

    #[test]
    pub fn test_commit_to_too_long_poly() {
        commit_to_too_long_poly_returns_err::<MclBackend>(&generate_trusted_setup);
    }

    #[test]
    pub fn test_proof_multi() {
        proof_multi::<MclBackend>(&generate_trusted_setup);
    }
}
