#[cfg(test)]
mod tests {
    use kzg_bench::tests::fft_fr::*;
    use rust_kzg_zkcrypto::fft_fr::{fft_fr_fast, fft_fr_slow};
    use rust_kzg_zkcrypto::fftsettings::ZkFFTSettings;
    use rust_kzg_zkcrypto::zkfr::blsScalar;

    #[test]
    fn compare_sft_fft_() {
        compare_sft_fft::<blsScalar, ZkFFTSettings>(&fft_fr_slow, &fft_fr_fast);
    }

    #[test]
    fn roundtrip_fft_() {
        roundtrip_fft::<blsScalar, ZkFFTSettings>();
    }

    //&fft_fr
    #[test]
    fn inverse_fft_() {
        inverse_fft::<blsScalar, ZkFFTSettings>();
    }

    #[test]
    fn stride_fft_() {
        stride_fft::<blsScalar, ZkFFTSettings>();
    }
}
