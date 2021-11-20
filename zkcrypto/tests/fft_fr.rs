#[cfg(test)]
pub mod tests {
    use kzg_bench::tests::fft_fr::*;
    use zkcrypto::fft_fr::{fft_fr, fft_fr_fast, fft_fr_slow};
	use zkcrypto::zkfr::blsScalar;
    use zkcrypto::fftsettings::ZkFFTSettings;

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
