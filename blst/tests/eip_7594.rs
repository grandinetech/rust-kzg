#[cfg(test)]
mod tests {
    use kzg::eip_4844::{bytes_to_blob, CELLS_PER_EXT_BLOB, FIELD_ELEMENTS_PER_CELL};
    use kzg_bench::tests::{eip_4844::generate_random_blob_bytes, eip_7594::{
        test_vectors_compute_cells_and_kzg_proofs, test_vectors_recover_cells_and_kzg_proofs,
        test_vectors_verify_cell_kzg_proof_batch,
    }, utils::get_trusted_setup_path};
    use rust_kzg_blst::{
        eip_4844::load_trusted_setup_filename_rust,
        eip_7594::{
            compute_cells_and_kzg_proofs_rust, recover_cells_and_kzg_proofs_rust,
            verify_cell_kzg_proof_batch_rust,
        },
        types::{
            fft_settings::FsFFTSettings,
            fp::FsFp,
            fr::FsFr,
            g1::{FsG1, FsG1Affine},
            g2::FsG2,
            kzg_settings::FsKZGSettings,
            poly::FsPoly,
        },
    };

    #[test]
    pub fn test_vectors_compute_cells_and_kzg_proofs_() {
        test_vectors_compute_cells_and_kzg_proofs::<
            FsFr,
            FsG1,
            FsG2,
            FsPoly,
            FsFFTSettings,
            FsKZGSettings,
            FsFp,
            FsG1Affine,
        >(
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

    #[test]
    pub fn test_recover_cells_and_kzg_proofs_succeeds_random_blob() {
        let s = load_trusted_setup_filename_rust(get_trusted_setup_path().as_str()).unwrap();
        let mut rng = rand::thread_rng();
        
        /* Get a random blob */
        let blob_bytes = generate_random_blob_bytes(&mut rng);
        let blob = bytes_to_blob(&blob_bytes).unwrap();
        
        let mut cells = vec![ core::array::from_fn::<_, FIELD_ELEMENTS_PER_CELL, _>(|_| FsFr::default()); CELLS_PER_EXT_BLOB ];
        let mut proofs = vec![FsG1::default(); CELLS_PER_EXT_BLOB];
        
        /* Get the cells and proofs */
        let mut result = compute_cells_and_kzg_proofs_rust(Some(&mut cells), Some(&mut proofs), &blob, &s);
        assert!(result.is_ok());

        let cell_indices: Vec<usize>= (0..).step_by(2).take(CELLS_PER_EXT_BLOB / 2).collect();
        let mut partial_cells = vec![ core::array::from_fn::<_, FIELD_ELEMENTS_PER_CELL, _>(|_| FsFr::default()); CELLS_PER_EXT_BLOB / 2 ];

        /* Erase half of the cells */
        for i in 0..(CELLS_PER_EXT_BLOB / 2) {
            partial_cells[i] = cells[cell_indices[i]].clone();
        }

        let mut recovered_cells = vec![ core::array::from_fn::<_, FIELD_ELEMENTS_PER_CELL, _>(|_| FsFr::default()); CELLS_PER_EXT_BLOB ];
        let mut recovered_proofs = vec![FsG1::default(); CELLS_PER_EXT_BLOB];

        /* Reconstruct with half of the cells */
        result = recover_cells_and_kzg_proofs_rust(&mut recovered_cells, Some(&mut recovered_proofs), &cell_indices, &mut partial_cells, &s);
        assert!(result.is_ok());

        /* Check that all of the cells match */
        assert!(recovered_cells == cells, "Cells do not match");
        assert!(recovered_proofs == proofs, "Proofs do not match");
    }
}
