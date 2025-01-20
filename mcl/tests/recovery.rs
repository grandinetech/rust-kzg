// #[path = "./local_tests/local_recovery.rs"]
// pub mod local_recovery;

#[cfg(test)]
mod tests {
    use kzg_bench::tests::recover::*;
    // uncomment to use the local tests
    //use crate::local_recovery::{recover_random, recover_simple};

    use rust_kzg_mcl::types::fft_settings::MclFFTSettings;
    use rust_kzg_mcl::types::fr::MclFr;
    use rust_kzg_mcl::types::poly::MclPoly;

    // Shared tests
    #[test]
    fn recover_simple_() {
        recover_simple::<MclFr, MclFFTSettings, MclPoly, MclPoly>();
    }

    #[test]
    fn recover_random_() {
        recover_random::<MclFr, MclFFTSettings, MclPoly, MclPoly>();
    }

    #[test]
    fn more_than_half_missing_() {
        more_than_half_missing::<MclFr, MclFFTSettings, MclPoly, MclPoly>();
    }
}
