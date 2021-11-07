#[cfg(test)]
mod tests {
    use kzg_bench::tests::kzg_proofs::*;
    use ckzg::kzgsettings::{KzgKZGSettings, generate_trusted_setup};
    use ckzg::fftsettings::KzgFFTSettings;
    use ckzg::consts::{BlstP1, BlstP2};
    use ckzg::finite::BlstFr;
    use ckzg::poly::KzgPoly;

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
