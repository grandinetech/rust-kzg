#[cfg(test)]
mod tests {
    use kzg_bench::tests::das::{das_extension_test_known, das_extension_test_random};
    use rust_kzg_arkworks3::kzg_proofs::LFFTSettings;
    use rust_kzg_arkworks3::kzg_types::ArkFr;

    #[test]
    fn das_extension_test_known_() {
        das_extension_test_known::<ArkFr, LFFTSettings>();
    }

    #[test]
    fn das_extension_test_random_() {
        das_extension_test_random::<ArkFr, LFFTSettings>();
    }
}
