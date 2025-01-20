#[cfg(test)]
mod tests {
    use kzg::G1;
    use kzg_bench::tests::fft_g1::{compare_ft_fft, roundtrip_fft, stride_fft};
    use rust_kzg_mcl::consts::G1_GENERATOR;
    use rust_kzg_mcl::fft_g1::{fft_g1_fast, fft_g1_slow};
    use rust_kzg_mcl::types::fft_settings::MclFFTSettings;
    use rust_kzg_mcl::types::fr::MclFr;
    use rust_kzg_mcl::types::g1::MclG1;

    fn make_data(n: usize) -> Vec<MclG1> {
        if n == 0 {
            return Vec::new();
        }
        let mut result: Vec<MclG1> = vec![MclG1::default(); n];
        result[0] = G1_GENERATOR;
        for i in 1..n {
            result[i] = result[i - 1].add_or_dbl(&G1_GENERATOR)
        }

        result
    }

    #[test]
    fn roundtrip_fft_() {
        roundtrip_fft::<MclFr, MclG1, MclFFTSettings>(&make_data);
    }

    #[test]
    fn stride_fft_() {
        stride_fft::<MclFr, MclG1, MclFFTSettings>(&make_data);
    }

    #[test]
    fn compare_sft_fft_() {
        compare_ft_fft::<MclFr, MclG1, MclFFTSettings>(&fft_g1_slow, &fft_g1_fast, &make_data);
    }
}
