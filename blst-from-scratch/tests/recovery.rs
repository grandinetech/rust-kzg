#[path = "./local_tests/local_recovery.rs"]
pub mod local_recovery;

#[cfg(test)]
pub mod tests {
    // use crate::local_recovery::{recover_simple, recover_random};
    // use blst_from_scratch::recovery::{recover_poly_from_samples};

    use kzg_bench::tests::recover::{recover_random, recover_simple};

    use blst_from_scratch::types::fft_settings::FsFFTSettings;
    use blst_from_scratch::types::fr::FsFr;
    use blst_from_scratch::types::poly::FsPoly;

    #[test]
    fn recover_simple_() {
        recover_simple::<FsFr, FsFFTSettings, FsPoly, FsPoly>()
    }

    #[test]
    fn recover_random_() {
        recover_random::<FsFr, FsFFTSettings, FsPoly, FsPoly>()
    }

// uncomment to run local tests
    // #[test]
    // fn recover_simple_() {
    //     recover_simple::<FsFr, FsFFTSettings, FsPoly>(&recover_poly_from_samples)
    // }

    // #[test]
    // fn recover_random_() {
    //     recover_random::<FsFr, FsFFTSettings, FsPoly>(&recover_poly_from_samples)
    // }
}