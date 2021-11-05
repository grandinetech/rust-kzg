#[cfg(test)]
mod tests {
    use kzg_bench::tests::kzg_proofs::proof_single;
    use kzg_bindings::kzgsettings::{KzgKZGSettings, generate_trusted_setup};
    use kzg_bindings::fftsettings::{KzgFFTSettings};
    use kzg_bindings::consts::{BlstP1, BlstP2};
    use kzg_bindings::finite::BlstFr;
    use kzg_bindings::poly::KzgPoly;

    #[test]
    fn test_proof_single() {
        proof_single::<BlstFr, BlstP1, BlstP2, KzgPoly, KzgFFTSettings, KzgKZGSettings>(&generate_trusted_setup);
    }
}
