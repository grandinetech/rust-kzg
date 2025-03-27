#[cfg(test)]
mod tests {

    use kzg_bench::tests::kzg_proofs::{
        commit_to_nil_poly, commit_to_too_long_poly_returns_err, proof_multi, proof_single,
        trusted_setup_in_correct_form,
    };

    use rust_kzg_constantine::{eip_7594::CtBackend, utils::generate_trusted_setup};

    #[test]
    pub fn test_trusted_setup_in_correct_form() {
        trusted_setup_in_correct_form::<CtBackend>(&generate_trusted_setup);
    }

    #[test]
    pub fn test_proof_single() {
        proof_single::<CtBackend>(&generate_trusted_setup);
    }

    #[test]
    pub fn test_commit_to_nil_poly() {
        commit_to_nil_poly::<CtBackend>(&generate_trusted_setup);
    }

    #[test]
    pub fn test_commit_to_too_long_poly() {
        commit_to_too_long_poly_returns_err::<CtBackend>(&generate_trusted_setup);
    }

    #[test]
    pub fn test_proof_multi() {
        proof_multi::<CtBackend>(&generate_trusted_setup);
    }
}
