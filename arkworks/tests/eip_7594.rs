#[cfg(test)]
mod tests {
    use kzg::eip_4844::bytes_to_blob;
    use kzg_bench::tests::eip_7594::{
        test_vectors_compute_cells_and_kzg_proofs, test_vectors_recover_cells_and_kzg_proofs,
        test_vectors_verify_cell_kzg_proof_batch,
    };
    use rust_kzg_arkworks::eip_4844::load_trusted_setup_filename_rust;
    use rust_kzg_arkworks::eip_7594::{
        compute_cells_and_kzg_proofs_rust, recover_cells_and_kzg_proofs_rust,
        verify_cell_kzg_proof_batch_rust,
    };
    use rust_kzg_arkworks::kzg_proofs::{FFTSettings as LFFTSettings, KZGSettings as LKZGSettings};
    use rust_kzg_arkworks::kzg_types::{ArkFp, ArkFr, ArkG1, ArkG1Affine, ArkG2};
    use rust_kzg_arkworks::utils::PolyData;

    #[test]
    pub fn test_vectors_compute_cells_and_kzg_proofs_() {
        test_vectors_compute_cells_and_kzg_proofs::<
            ArkFr,
            ArkG1,
            ArkG2,
            PolyData,
            LFFTSettings,
            LKZGSettings,
            ArkFp,
            ArkG1Affine,
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
}
