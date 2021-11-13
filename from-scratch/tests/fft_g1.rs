#[cfg(test)]
pub mod tests {
    use kzg::G1;
    use kzg_bench::tests::fft_g1::{compare_ft_fft, roundtrip_fft, stride_fft};
    use kzg_from_scratch::consts::G1_GENERATOR;
    use kzg_from_scratch::fft_g1::{fft_g1_fast, fft_g1_slow};
    use kzg_from_scratch::kzg_types::{FsFFTSettings, FsFr, FsG1};

    fn make_data(n: usize) -> Vec<FsG1> {
        if n == 0 {
            return Vec::default();
        }
        let mut result: Vec<FsG1> = vec![FsG1::default(); n];
        result[0] = G1_GENERATOR;
        for i in 1..n {
            result[i] = result[i - 1].add_or_dbl(&G1_GENERATOR)
        }
        return result;
    }

    #[test]
    fn roundtrip_fft_() {
        roundtrip_fft::<FsFr, FsG1, FsFFTSettings>(&make_data);
    }

    #[test]
    fn stride_fft_() {
        stride_fft::<FsFr, FsG1, FsFFTSettings>(&make_data);
    }

    #[test]
    fn compare_sft_fft_() {
        compare_ft_fft::<FsFr, FsG1, FsFFTSettings>(&fft_g1_slow, &fft_g1_fast, &make_data);
    }
}
