#[cfg(test)]
mod tests {
    use kzg_bench::tests::recover::*;
    use ckzg::fftsettings::KzgFFTSettings;
    use ckzg::finite::BlstFr;
    use ckzg::poly::KzgPoly;

    #[test]
    fn test_recover_simple() {
        recover_simple::<BlstFr, KzgFFTSettings, KzgPoly, KzgPoly>();
    }

    #[test]
    fn test_recover_random() {
        recover_random::<BlstFr, KzgFFTSettings, KzgPoly, KzgPoly>();
    }
}
