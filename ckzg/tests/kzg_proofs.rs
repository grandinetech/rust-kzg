#[cfg(test)]
mod tests {
    use kzg_bench::tests::kzg_proofs::*;
    use kzg_bindings::kzgsettings::{KzgKZGSettings, generate_trusted_setup};
    use kzg_bindings::fftsettings::{KzgFFTSettings};
    use kzg_bindings::consts::{BlstP1, BlstP2};
    use kzg_bindings::finite::BlstFr;
    use kzg_bindings::poly::KzgPoly;

    #[test]
    fn test_proof_single() {
        proof_single::<BlstFr, BlstP1, BlstP2, KzgPoly, KzgFFTSettings, KzgKZGSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_commit_to_nil_poly() {
        commit_to_nil_poly::<BlstFr, BlstP1, BlstP2, KzgPoly, KzgFFTSettings, KzgKZGSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_commit_to_too_long_poly() {
        commit_to_too_long_poly::<BlstFr, BlstP1, BlstP2, KzgPoly, KzgFFTSettings, KzgKZGSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_proof_multi() {
        proof_multi::<BlstFr, BlstP1, BlstP2, KzgPoly, KzgFFTSettings, KzgKZGSettings>(&generate_trusted_setup);
    }
}
