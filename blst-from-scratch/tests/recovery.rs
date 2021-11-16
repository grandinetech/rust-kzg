#[path = "./local_tests/local_recovery.rs"]
pub mod local_recovery;

#[cfg(test)]
pub mod tests {
    use crate::local_recovery::{recover_simple, recover_random};
    use kzg_from_scratch::recovery::{recover_poly_from_samples};
    use kzg_from_scratch::kzg_types::{FsFr, FsPoly, FsFFTSettings};

    #[test]
    fn recover_simple_() {
        recover_simple::<FsFr, FsFFTSettings, FsPoly>(&recover_poly_from_samples)
    }

    #[test]
    fn recover_random_() {
        recover_random::<FsFr, FsFFTSettings, FsPoly>(&recover_poly_from_samples)
    }
}