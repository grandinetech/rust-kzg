#[cfg(test)]
pub mod tests {
    use kzg_from_scratch::kzg_types::{FsFr, FsG1, FsFFTSettings};
    use kzg_bench::tests::fft_g1::{roundtrip_fft, stride_fft};
    use kzg_from_scratch::consts::{G1_GENERATOR};
    use kzg::G1;

    fn make_data(n: usize) -> Vec<FsG1> {
        if n == 0 {
            return Vec::default();
        }
        let mut result: Vec<FsG1> = vec![FsG1::default(); n];
        result[0] = G1_GENERATOR;
        for i in 1..n {
            result[i] = result[i-1].add_or_double(&G1_GENERATOR)
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

}
