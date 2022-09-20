#[cfg(test)]
pub mod tests {
    use kzg::G1;
    use kzg_bench::tests::fft_g1::{compare_ft_fft, roundtrip_fft, stride_fft};
    use zkcrypto::fft_g1::{fft_g1_fast, fft_g1_slow};
    use zkcrypto::fftsettings::ZkFFTSettings;
    use zkcrypto::kzg_types::ZkG1Projective;
    use zkcrypto::kzg_types::G1_GENERATOR;
    use zkcrypto::zkfr::blsScalar;

    fn make_data(n: usize) -> Vec<ZkG1Projective> {
        if n == 0 {
            return Vec::default();
        }
        let mut result: Vec<ZkG1Projective> = vec![Default::default(); n];
        result[0] = G1_GENERATOR;
        for i in 1..n {
            result[i] = result[i - 1].add_or_dbl(&G1_GENERATOR)
        }
        result
    }

    #[test]
    fn roundtrip_fft_() {
        roundtrip_fft::<blsScalar, ZkG1Projective, ZkFFTSettings>(&make_data);
    }

    #[test]
    fn stride_fft_() {
        stride_fft::<blsScalar, ZkG1Projective, ZkFFTSettings>(&make_data);
    }

    #[test]
    fn compare_sft_fft_() {
        compare_ft_fft::<blsScalar, ZkG1Projective, ZkFFTSettings>(
            &fft_g1_slow,
            &fft_g1_fast,
            &make_data,
        );
    }
}
