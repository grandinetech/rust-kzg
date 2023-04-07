#[cfg(test)]
mod tests {
    use ckzg::fftsettings::KzgFFTSettings;
    use ckzg::finite::BlstFr;
    use ckzg::poly::KzgPoly;
    use kzg_bench::tests::recover::*;

    #[test]
    fn test_recover_simple() {
        recover_simple::<BlstFr, KzgFFTSettings, KzgPoly, KzgPoly>();
    }

    #[test]
    fn test_recover_random() {
        recover_random::<BlstFr, KzgFFTSettings, KzgPoly, KzgPoly>();
    }

    #[test]
    fn more_than_half_missing_() {
        more_than_half_missing::<BlstFr, KzgFFTSettings, KzgPoly, KzgPoly>();
    }
}
