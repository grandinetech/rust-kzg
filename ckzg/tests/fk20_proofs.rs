#[cfg(test)]
mod tests {
    use kzg_bench::tests::fk20_proofs::*;

    use ckzg::consts::{BlstP1, BlstP2};
    use ckzg::fftsettings::KzgFFTSettings;
    use ckzg::fk20settings::KzgFK20SingleSettings;
    use ckzg::kzgsettings::{KzgKZGSettings, generate_trusted_setup};
    use ckzg::finite::BlstFr;
    use ckzg::poly::KzgPoly;
    use ckzg::utils::reverse_bits_limited;

    #[test]
    fn test_fk_single_strided() {
        fk_single_strided::<BlstFr, BlstP1, BlstP2, KzgPoly, KzgFFTSettings, KzgKZGSettings, KzgFK20SingleSettings>(&generate_trusted_setup, &reverse_bits_limited);
    }
}
