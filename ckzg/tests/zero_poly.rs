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
        reduce_partials_random::<BlstFr, KzgFFTSettings, KzgPoly>();
    }

    #[test]
    fn test_check_test_data() {
        check_test_data::<BlstFr, KzgFFTSettings, KzgPoly>();
    }

    #[test]
    fn test_zero_poly_random() {
        zero_poly_random::<BlstFr, KzgFFTSettings, KzgPoly>();
    }

    #[test]
    fn test_zero_poly_known() {
        zero_poly_known::<BlstFr, KzgFFTSettings, KzgPoly>();
    }

    #[test]
    fn test_zero_poly_all_but_one() {
        zero_poly_all_but_one::<BlstFr, KzgFFTSettings, KzgPoly>();
    }

    #[test]
    fn test_zero_poly_252() {
        zero_poly_252::<BlstFr, KzgFFTSettings, KzgPoly>();
    }
}
