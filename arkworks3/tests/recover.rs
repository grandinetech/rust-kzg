#[cfg(test)]
mod recover_tests {
    use kzg_bench::tests::recover::*;
    use rust_kzg_arkworks3::kzg_proofs::LFFTSettings as FFTSettings;
    use rust_kzg_arkworks3::kzg_types::ArkFr as Fr;
    use rust_kzg_arkworks3::utils::PolyData;

    #[test]
    fn recover_simple_() {
        recover_simple::<Fr, FFTSettings, PolyData, PolyData>();
    }

    //Could be not working because of zero poly.
    #[test]
    fn recover_random_() {
        recover_random::<Fr, FFTSettings, PolyData, PolyData>();
    }

    #[test]
    fn more_than_half_missing_() {
        more_than_half_missing::<Fr, FFTSettings, PolyData, PolyData>();
    }
}
