#[cfg(test)]
mod tests {
    use kzg_bench::tests::fk20_proofs::*;

    use ckzg::consts::{BlstP1, BlstP2};
    use ckzg::fftsettings::KzgFFTSettings;
    use ckzg::fk20settings::{KzgFK20MultiSettings, KzgFK20SingleSettings};
    use ckzg::kzgsettings::{KzgKZGSettings, generate_trusted_setup};
    use ckzg::finite::BlstFr;
    use ckzg::poly::KzgPoly;

    #[test]
    fn test_fk_single() {
        fk_single::<BlstFr, BlstP1, BlstP2, KzgPoly, KzgFFTSettings, KzgKZGSettings, KzgFK20SingleSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_single_strided() {
        fk_single_strided::<BlstFr, BlstP1, BlstP2, KzgPoly, KzgFFTSettings, KzgKZGSettings, KzgFK20SingleSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_settings() {
        fk_multi_settings::<BlstFr, BlstP1, BlstP2, KzgPoly, KzgFFTSettings, KzgKZGSettings, KzgFK20MultiSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_1_512() {
        fk_multi_chunk_len_1_512::<BlstFr, BlstP1, BlstP2, KzgPoly, KzgFFTSettings, KzgKZGSettings, KzgFK20MultiSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_16_512() {
        fk_multi_chunk_len_16_512::<BlstFr, BlstP1, BlstP2, KzgPoly, KzgFFTSettings, KzgKZGSettings, KzgFK20MultiSettings>(&generate_trusted_setup);
    }

    #[test]
    fn test_fk_multi_chunk_len_16_16() {
        fk_multi_chunk_len_16_16::<BlstFr, BlstP1, BlstP2, KzgPoly, KzgFFTSettings, KzgKZGSettings, KzgFK20MultiSettings>(&generate_trusted_setup);
    }
}
