#[cfg(test)]
mod tests {
    use kzg::eip_4844::bytes_to_blob;
    use kzg_bench::tests::eip_7594::{
        test_vectors_compute_cells_and_kzg_proofs, test_vectors_recover_cells_and_kzg_proofs,
        test_vectors_verify_cell_kzg_proof_batch,
    };
    use rust_kzg_blst::{
        eip_4844::load_trusted_setup_filename_rust,
        eip_7594::{
            compute_cells_and_kzg_proofs, recover_cells_and_kzg_proofs, verify_cell_kzg_proof_batch,
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
            &compute_cells_and_kzg_proofs,
            &bytes_to_blob,
        );
    }

    #[test]
    pub fn test_vectors_recover_cells_and_kzg_proofs_() {
        test_vectors_recover_cells_and_kzg_proofs(
            &load_trusted_setup_filename_rust,
            &recover_cells_and_kzg_proofs,
        );
    }

    #[test]
    pub fn test_vectors_verify_cell_kzg_proof_batch_() {
        test_vectors_verify_cell_kzg_proof_batch(
            &load_trusted_setup_filename_rust,
            &verify_cell_kzg_proof_batch,
        );
    }
}
