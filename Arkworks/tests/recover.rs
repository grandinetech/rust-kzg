#[cfg(test)]
pub mod recover_tests {
    use kzg_bench::tests::recover::*;
    use arkworks::kzg_types::FsFr as Fr;
    use arkworks::utils::PolyData;
    use arkworks::kzg_proofs::FFTSettings;

    #[test]
    fn recover_simple_() {
        recover_simple::<Fr, FFTSettings, PolyData, PolyData>();
    }

    //Could be not working because of zero poly.
    #[test]
    fn recover_random_() {
        recover_random::<Fr, FFTSettings, PolyData, PolyData>();
    }
}