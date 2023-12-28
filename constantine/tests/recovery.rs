// #[path = "./local_tests/local_recovery.rs"]
// pub mod local_recovery;

#[cfg(test)]
mod tests {
    use kzg_bench::tests::recover::*;
    // uncomment to use the local tests
    //use crate::local_recovery::{recover_random, recover_simple};

    use rust_kzg_constantine::types::fft_settings::CtFFTSettings;
    use rust_kzg_constantine::types::fr::CtFr;
    use rust_kzg_constantine::types::poly::CtPoly;

    // Shared tests
    #[test]
    fn recover_simple_() {
        recover_simple::<CtFr, CtFFTSettings, CtPoly, CtPoly>();
    }

    #[test]
    fn recover_random_() {
        recover_random::<CtFr, CtFFTSettings, CtPoly, CtPoly>();
    }

    #[test]
    fn more_than_half_missing_() {
        more_than_half_missing::<CtFr, CtFFTSettings, CtPoly, CtPoly>();
    }
}
