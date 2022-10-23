#[cfg(test)]

mod tests {

    use ckzg::{eip_4844::{compute_powers, bytes_to_bls_field}, finite::BlstFr};
    use kzg_bench::tests::eip_4844::{
        compute_powers_test, bytes_to_bls_field_test
    };

    #[test]
    pub fn bytes_to_bls_field_test_() {
        bytes_to_bls_field_test::<BlstFr>(&bytes_to_bls_field);
    }

    #[test]
    pub fn compute_powers_test_() {
        compute_powers_test::<BlstFr>(&bytes_to_bls_field, &compute_powers);
    }

}
