#[cfg(test)]
mod tests {
    use kzg_bench::tests::c_bindings::{
        blob_to_kzg_commitment_invalid_blob_test, load_trusted_setup_file_invalid_format_test,
        load_trusted_setup_file_valid_format_test, load_trusted_setup_invalid_form_test,
        load_trusted_setup_invalid_g1_byte_length_test, load_trusted_setup_invalid_g1_point_test,
        load_trusted_setup_invalid_g2_byte_length_test, load_trusted_setup_invalid_g2_point_test,
    };
    use rust_kzg_blst::eip_4844::{
        blob_to_kzg_commitment, load_trusted_setup, load_trusted_setup_file,
    };

    #[test]
    fn blob_to_kzg_commitment_invalid_blob() {
        blob_to_kzg_commitment_invalid_blob_test(load_trusted_setup, blob_to_kzg_commitment);
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
}
