#[cfg(test)]
mod tests {
    use kzg::{
        eip_4844::{blob_to_kzg_commitment_rust, bytes_to_blob},
        eth, DAS,
    };
    use kzg_bench::tests::{
        eip_4844::generate_random_blob_bytes,
        eip_7594::{
            test_vectors_compute_cells_and_kzg_proofs, test_vectors_recover_cells_and_kzg_proofs,
            test_vectors_verify_cell_kzg_proof_batch,
        },
        utils::get_trusted_setup_path,
    };
    use rust_kzg_mcl::{
        eip_4844::load_trusted_setup_filename_rust,
        eip_7594::MclBackend,
        types::{fr::MclFr, g1::MclG1, kzg_settings::MclKZGSettings},
    };

    #[test]
    pub fn test_vectors_compute_cells_and_kzg_proofs_() {
        test_vectors_compute_cells_and_kzg_proofs::<MclBackend>(
            &load_trusted_setup_filename_rust,
            &bytes_to_blob,
        );
    }

    #[test]
    pub fn test_vectors_recover_cells_and_kzg_proofs_() {
        test_vectors_recover_cells_and_kzg_proofs::<MclBackend>(&load_trusted_setup_filename_rust);
    }

    #[test]
    pub fn test_vectors_verify_cell_kzg_proof_batch_() {
        test_vectors_verify_cell_kzg_proof_batch::<MclBackend>(&load_trusted_setup_filename_rust);
    }

    #[test]
    pub fn test_recover_cells_and_kzg_proofs_succeeds_random_blob() {
        let settings = load_trusted_setup_filename_rust(get_trusted_setup_path().as_str()).unwrap();
        let mut rng = rand::thread_rng();

        /* Get a random blob */
        let blob_bytes = generate_random_blob_bytes(&mut rng);
        let blob: Vec<MclFr> = bytes_to_blob(&blob_bytes).unwrap();

        let mut cells =
            vec![MclFr::default(); eth::CELLS_PER_EXT_BLOB * eth::FIELD_ELEMENTS_PER_CELL];
        let mut proofs = vec![MclG1::default(); eth::CELLS_PER_EXT_BLOB];

        /* Get the cells and proofs */
        let mut result = <MclKZGSettings as DAS<MclBackend>>::compute_cells_and_kzg_proofs(
            &settings,
            Some(&mut cells),
            Some(&mut proofs),
            &blob,
        );
        assert!(result.is_ok());

        let cell_indices: Vec<usize> = (0..).step_by(2).take(eth::CELLS_PER_EXT_BLOB / 2).collect();
        let mut partial_cells =
            vec![MclFr::default(); (eth::CELLS_PER_EXT_BLOB / 2) * eth::FIELD_ELEMENTS_PER_CELL];

        /* Erase half of the cells */
        for i in 0..(eth::CELLS_PER_EXT_BLOB / 2) {
            partial_cells[i * eth::FIELD_ELEMENTS_PER_CELL..(i + 1) * eth::FIELD_ELEMENTS_PER_CELL]
                .clone_from_slice(
                    &cells[cell_indices[i] * eth::FIELD_ELEMENTS_PER_CELL
                        ..(cell_indices[i] + 1) * eth::FIELD_ELEMENTS_PER_CELL],
                );
        }

        let mut recovered_cells =
            vec![MclFr::default(); eth::CELLS_PER_EXT_BLOB * eth::FIELD_ELEMENTS_PER_CELL];
        let mut recovered_proofs = vec![MclG1::default(); eth::CELLS_PER_EXT_BLOB];

        /* Reconstruct with half of the cells */
        result = <MclKZGSettings as DAS<MclBackend>>::recover_cells_and_kzg_proofs(
            &settings,
            &mut recovered_cells,
            Some(&mut recovered_proofs),
            &cell_indices,
            &partial_cells,
        );
        assert!(result.is_ok());

        /* Check that all of the cells match */
        assert!(recovered_cells == cells, "Cells do not match");
        assert!(recovered_proofs == proofs, "Proofs do not match");
    }

    #[test]
    pub fn test_verify_cell_kzg_proof_batch_succeeds_random_blob() {
        let settings = load_trusted_setup_filename_rust(get_trusted_setup_path().as_str()).unwrap();
        let mut rng = rand::thread_rng();

        /* Get a random blob */
        let blob_bytes = generate_random_blob_bytes(&mut rng);
        let blob = bytes_to_blob(&blob_bytes).unwrap();

        /* Get the commitment to the blob */
        let commitment_result = blob_to_kzg_commitment_rust(&blob, &settings);
        assert!(commitment_result.is_ok());
        let commitment = commitment_result.unwrap();

        let mut cells: Vec<MclFr> =
            vec![MclFr::default(); eth::CELLS_PER_EXT_BLOB * eth::FIELD_ELEMENTS_PER_CELL];
        let mut proofs = vec![MclG1::default(); eth::CELLS_PER_EXT_BLOB];

        /* Compute cells and proofs */
        let result = <MclKZGSettings as DAS<MclBackend>>::compute_cells_and_kzg_proofs(
            &settings,
            Some(&mut cells),
            Some(&mut proofs),
            &blob,
        );
        assert!(result.is_ok());

        /* Initialize list of commitments & cell indices */
        let commitments = vec![commitment; eth::CELLS_PER_EXT_BLOB];

        let cell_indices: Vec<usize> = (0..).step_by(1).take(eth::CELLS_PER_EXT_BLOB).collect();

        /* Verify all the proofs */
        let verify_result = <MclKZGSettings as DAS<MclBackend>>::verify_cell_kzg_proof_batch(
            &settings,
            &commitments,
            &cell_indices,
            &cells,
            &proofs,
        );
        assert!(verify_result.is_ok());
    }
}
