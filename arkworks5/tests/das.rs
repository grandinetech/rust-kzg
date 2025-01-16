#[cfg(test)]
mod tests {
    use kzg_bench::tests::das::{das_extension_test_known, das_extension_test_random};
    use rust_kzg_arkworks5::kzg_proofs::FFTSettings;
    use rust_kzg_arkworks5::kzg_types::ArkFr;

    #[test]
    fn das_extension_test_known_() {
        das_extension_test_known::<ArkFr, FFTSettings>();
    }

    #[test]
    fn das_extension_test_random_() {
        das_extension_test_random::<ArkFr, FFTSettings>();
    }
}
