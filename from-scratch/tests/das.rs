#[cfg(test)]
pub mod tests {
    use kzg_bench::tests::das::{das_extension_test_known, das_extension_test_random};
    use kzg_from_scratch::kzg_types::{FsFFTSettings, FsFr};

    #[test]
    fn das_extension_test_known_() {
        das_extension_test_known::<FsFr, FsFFTSettings>();
    }

    #[test]
    fn das_extension_test_random_() {
        das_extension_test_random::<FsFr, FsFFTSettings>();
    }
}
