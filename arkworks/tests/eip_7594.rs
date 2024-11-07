
#[cfg(test)]
mod tests {
    use kzg::eip_4844::bytes_to_blob;
    use kzg_bench::tests::eip_7594::{
        test_vectors_compute_cells_and_kzg_proofs, test_vectors_recover_cells_and_kzg_proofs,
        test_vectors_verify_cell_kzg_proof_batch,
    };
    use rust_kzg_constantine::{
        eip_4844::load_trusted_setup_filename_rust,
        eip_7594::{
            compute_cells_and_kzg_proofs_rust, recover_cells_and_kzg_proofs_rust,
            verify_cell_kzg_proof_batch_rust,
        },
    };

    #[test]
    pub fn test_vectors_compute_cells_and_kzg_proofs_() {
        test_vectors_compute_cells_and_kzg_proofs(
            &load_trusted_setup_filename_rust,
            &compute_cells_and_kzg_proofs_rust,
            &bytes_to_blob,
        );
    }

    #[test]
    pub fn test_vectors_recover_cells_and_kzg_proofs_() {
        test_vectors_recover_cells_and_kzg_proofs(
            &load_trusted_setup_filename_rust,
            &recover_cells_and_kzg_proofs_rust,
        );
    }

    #[test]
    pub fn test_vectors_verify_cell_kzg_proof_batch_() {
        test_vectors_verify_cell_kzg_proof_batch(
            &load_trusted_setup_filename_rust,
            &verify_cell_kzg_proof_batch_rust,
        );
    }
}