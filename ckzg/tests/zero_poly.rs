#[cfg(test)]
mod tests {
    use kzg_bench::tests::zero_poly::*;
    use ckzg::fftsettings::KzgFFTSettings;
    use ckzg::finite::BlstFr;
    use ckzg::poly::KzgPoly;

    #[test]
    fn reduce_partials() {
        test_reduce_partials::<BlstFr, KzgFFTSettings, KzgPoly>();
    }

    #[test]
    fn test_reduce_partials_random() {
        test_reduce_partials::<BlstFr, KzgFFTSettings, KzgPoly>();
    }

    #[test]
    fn test_zero_poly_random() {
        test_reduce_partials::<BlstFr, KzgFFTSettings, KzgPoly>();
    }

    #[test]
    fn test_zero_poly_known() {
        test_reduce_partials::<BlstFr, KzgFFTSettings, KzgPoly>();
    }

    #[test]
    fn test_zero_poly_all_but_one() {
        test_reduce_partials::<BlstFr, KzgFFTSettings, KzgPoly>();
    }

    #[test]
    fn test_zero_poly_252() {
        test_reduce_partials::<BlstFr, KzgFFTSettings, KzgPoly>();
    }
}
