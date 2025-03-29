#[cfg(test)]
mod tests {
    use kzg_bench::tests::kzg_proofs::{
        commit_to_nil_poly, commit_to_too_long_poly_returns_err, proof_multi, proof_single,
        trusted_setup_in_correct_form,
    };
    use rust_kzg_arkworks4::eip_7594::ArkBackend;
    use rust_kzg_arkworks4::kzg_proofs::generate_trusted_setup;

    #[test]
    pub fn test_trusted_setup_in_correct_form() {
        trusted_setup_in_correct_form::<ArkBackend>(&generate_trusted_setup);
    }

    #[test]
    fn proof_single_() {
        proof_single::<ArkBackend>(&generate_trusted_setup);
    }

    #[test]
    fn commit_to_nil_poly_() {
        commit_to_nil_poly::<ArkBackend>(&generate_trusted_setup);
    }

    #[test]
    fn commit_to_too_long_poly_() {
        commit_to_too_long_poly_returns_err::<ArkBackend>(&generate_trusted_setup);
    }

    #[test]
    fn proof_multi_() {
        proof_multi::<ArkBackend>(&generate_trusted_setup);
    }
}
