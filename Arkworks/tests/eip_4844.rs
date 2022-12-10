#[cfg(test)]
pub mod tests {
    use arkworks::eip_4844::{bytes_from_bls_field, bytes_to_bls_field};
    use kzg_bench::tests::eip_4844::{bytes_to_bls_field_test};
    use arkworks::kzg_types::FsFr;

    #[test]
    fn bytes_to_bls_field_test_() {
        bytes_to_bls_field_test::<FsFr>(&bytes_to_bls_field, &bytes_from_bls_field);
    }
}
