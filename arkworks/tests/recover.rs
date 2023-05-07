#[cfg(test)]
mod recover_tests {
    use arkworks::kzg_proofs::FFTSettings;
    use arkworks::kzg_types::FsFr as Fr;
    use arkworks::utils::PolyData;
    use kzg_bench::tests::recover::*;

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
