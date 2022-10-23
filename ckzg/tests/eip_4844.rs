#[cfg(test)]

mod tests {

    use ckzg::{eip_4844::{compute_powers, bytes_to_bls_field}, finite::BlstFr};
    use kzg_bench::tests::eip_4844::{
        compute_powers_test
    };


    #[test]
    pub fn compute_powers_test_() {
        compute_powers_test::<BlstFr>(&compute_powers, &bytes_to_bls_field);
    }

}
