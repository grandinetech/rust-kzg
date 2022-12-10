#[cfg(test)]
pub mod tests {
    use arkworks::eip_4844::{bytes_from_bls_field, bytes_to_bls_field, compute_powers};
    use kzg_bench::tests::eip_4844::{bytes_to_bls_field_test, compute_powers_test};
    use arkworks::kzg_types::FsFr;

    #[test]
    fn bytes_to_bls_field_test_() {
        bytes_to_bls_field_test::<FsFr>(&bytes_to_bls_field, &bytes_from_bls_field);
    }

    #[test]
    pub fn compute_powers_test_() {
        compute_powers_test::<FsFr>(&bytes_to_bls_field, &compute_powers);
    }
}
