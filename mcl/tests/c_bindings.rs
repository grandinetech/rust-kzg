#[cfg(test)]
mod tests {
    use kzg_bench::tests::c_bindings::{
        blob_to_kzg_commitment_invalid_blob_test,
        compute_blob_kzg_proof_commitment_is_point_at_infinity_test,
        compute_blob_kzg_proof_invalid_blob_test, free_trusted_setup_null_ptr_test,
        free_trusted_setup_set_all_values_to_null_test,
        load_trusted_setup_file_invalid_format_test, load_trusted_setup_file_valid_format_test,
        load_trusted_setup_invalid_form_test, load_trusted_setup_invalid_g1_byte_length_test,
        load_trusted_setup_invalid_g1_point_test, load_trusted_setup_invalid_g2_byte_length_test,
        load_trusted_setup_invalid_g2_point_test,
    };
    use rust_kzg_blst::eip_4844::{
        blob_to_kzg_commitment, compute_blob_kzg_proof, free_trusted_setup, load_trusted_setup,
        load_trusted_setup_file,
    };

    #[test]
    fn blob_to_kzg_commitment_invalid_blob() {
        blob_to_kzg_commitment_invalid_blob_test(blob_to_kzg_commitment, load_trusted_setup_file);
    }

    #[test]
    fn load_trusted_setup_invalid_g1_byte_length() {
        load_trusted_setup_invalid_g1_byte_length_test(load_trusted_setup);
    }

    #[test]
    fn load_trusted_setup_invalid_g1_point() {
        load_trusted_setup_invalid_g1_point_test(load_trusted_setup);
    }

    #[test]
    fn load_trusted_setup_invalid_g2_byte_length() {
        load_trusted_setup_invalid_g2_byte_length_test(load_trusted_setup);
    }

    #[test]
    fn load_trusted_setup_invalid_g2_point() {
        load_trusted_setup_invalid_g2_point_test(load_trusted_setup);
    }

    #[test]
    fn load_trusted_setup_invalid_form() {
        load_trusted_setup_invalid_form_test(load_trusted_setup);
    }

    #[test]
    fn load_trusted_setup_file_invalid_format() {
        load_trusted_setup_file_invalid_format_test(load_trusted_setup_file);
    }

    #[test]
    fn load_trusted_setup_file_valid_format() {
        load_trusted_setup_file_valid_format_test(load_trusted_setup_file);
    }

    #[test]
    fn free_trusted_setup_null_ptr() {
        free_trusted_setup_null_ptr_test(free_trusted_setup);
    }

    #[test]
    fn free_trusted_setup_set_all_values_to_null() {
        free_trusted_setup_set_all_values_to_null_test(free_trusted_setup, load_trusted_setup_file);
    }

    #[test]
    fn compute_blob_kzg_proof_invalid_blob() {
        compute_blob_kzg_proof_invalid_blob_test(compute_blob_kzg_proof, load_trusted_setup_file);
    }

    #[test]
    fn compute_blob_kzg_proof_commitment_is_point_at_infinity() {
        compute_blob_kzg_proof_commitment_is_point_at_infinity_test(
            compute_blob_kzg_proof,
            load_trusted_setup_file,
        );
    }
}
