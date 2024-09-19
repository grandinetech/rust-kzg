#[cfg(test)]
mod tests {
    use kzg::eip_4844::{bytes_to_blob, load_trusted_setup_rust};
    use kzg_bench::tests::eip_7594::test_vectors_compute_cells_and_kzg_proofs;
    use rust_kzg_blst::{
        eip_4844::{load_trusted_setup, load_trusted_setup_file, load_trusted_setup_filename_rust},
        eip_7594::compute_cells_and_kzg_proofs,
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
}
