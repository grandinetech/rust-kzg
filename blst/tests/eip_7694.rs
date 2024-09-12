#[cfg(test)]
mod tests {
    use kzg::eip_4844::{bytes_to_blob, compute_cells_and_kzg_proofs_rust};
    use kzg_bench::tests::eip_7594::test_vectors_compute_cells_and_kzg_proofs;
    use rust_kzg_blst::{
        eip_4844::load_trusted_setup,
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
    pub fn bytes_to_bls_field_test_() {
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
            &load_trusted_setup,
            &compute_cells_and_kzg_proofs_rust,
            &bytes_to_blob,
        );
    }
}
