#[cfg(test)]
mod tests {
    use kzg::G1;
    use kzg_bench::tests::fft_g1::{compare_ft_fft, roundtrip_fft, stride_fft};
    use rust_kzg_constantine::consts::G1_GENERATOR;
    use rust_kzg_constantine::fft_g1::{fft_g1_fast, fft_g1_slow};
    use rust_kzg_constantine::types::fft_settings::CtFFTSettings;
    use rust_kzg_constantine::types::fr::CtFr;
    use rust_kzg_constantine::types::g1::CtG1;

    fn make_data(n: usize) -> Vec<CtG1> {
        if n == 0 {
            return Vec::new();
        }
        let mut result: Vec<CtG1> = vec![CtG1::default(); n];
        result[0] = G1_GENERATOR;
        for i in 1..n {
            result[i] = result[i - 1].add_or_dbl(&G1_GENERATOR)
        }

        result
    }

    #[test]
    fn roundtrip_fft_() {
        roundtrip_fft::<CtFr, CtG1, CtFFTSettings>(&make_data);
    }

    #[test]
    fn stride_fft_() {
        stride_fft::<CtFr, CtG1, CtFFTSettings>(&make_data);
    }

    #[test]
    fn compare_sft_fft_() {
        compare_ft_fft::<CtFr, CtG1, CtFFTSettings>(&fft_g1_slow, &fft_g1_fast, &make_data);
    }
}
