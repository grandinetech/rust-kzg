#[cfg(test)]
pub mod tests {
    use arkworks::kzg_proofs::FFTSettings;
    use arkworks::kzg_types::FsFr;
    use kzg_bench::tests::das::{das_extension_test_known, das_extension_test_random};

    #[test]
    fn das_extension_test_known_() {
        das_extension_test_known::<FsFr, FFTSettings>();
    }

    #[test]
    fn das_extension_test_random_() {
        das_extension_test_random::<FsFr, FFTSettings>();
    }
}
