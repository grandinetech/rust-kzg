// #[path = "./local_tests/local_recovery.rs"]
// pub mod local_recovery;

#[cfg(test)]
mod tests {
    use kzg_bench::tests::recover::*;
    // uncomment to use the local tests
    //use crate::local_recovery::{recover_random, recover_simple};

    use blst_from_scratch::types::fft_settings::FsFFTSettings;
    use blst_from_scratch::types::fr::FsFr;
    use blst_from_scratch::types::poly::FsPoly;

    // Shared tests
    #[test]
    fn recover_simple_() {
        recover_simple::<FsFr, FsFFTSettings, FsPoly, FsPoly>();
    }

    #[test]
    fn recover_random_() {
        recover_random::<FsFr, FsFFTSettings, FsPoly, FsPoly>();
    }

    #[test]
    fn more_than_half_missing_() {
        more_than_half_missing::<FsFr, FsFFTSettings, FsPoly, FsPoly>();
    }
}
