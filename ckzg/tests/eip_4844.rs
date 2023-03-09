#[cfg(test)]

mod tests {

    use ckzg::{
        consts::{BlstP1, BlstP2},
        eip_4844::{
            blob_to_kzg_commitment_rust, compute_aggregate_kzg_proof_rust, load_trusted_setup_rust,
            verify_aggregate_kzg_proof_rust,
        },
        fftsettings4844::KzgFFTSettings4844,
        finite::BlstFr,
        kzgsettings4844::KzgKZGSettings4844,
        poly::KzgPoly,
    };
    use kzg_bench::tests::eip_4844::{
        aggregate_proof_for_single_blob_test, eip4844_test, verify_aggregate_kzg_proof_test_empty,
    };

    #[test]
    pub fn eip4844_test_() {
        eip4844_test::<BlstFr, BlstP1, BlstP2, KzgPoly, KzgFFTSettings4844, KzgKZGSettings4844>(
            &load_trusted_setup_rust,
            &blob_to_kzg_commitment_rust,
            &compute_aggregate_kzg_proof_rust,
            &verify_aggregate_kzg_proof_rust,
        );
    }

    #[test]
    pub fn verify_aggregate_kzg_proof_test_empty_() {
        verify_aggregate_kzg_proof_test_empty::<
            BlstFr,
            BlstP1,
            BlstP2,
            KzgPoly,
            KzgFFTSettings4844,
            KzgKZGSettings4844,
        >(
            &load_trusted_setup_rust,
            &compute_aggregate_kzg_proof_rust,
            &verify_aggregate_kzg_proof_rust,
        );
    }

    #[test]
    pub fn aggregate_proof_for_single_blob_test_() {
        aggregate_proof_for_single_blob_test::<
            BlstFr,
            BlstP1,
            BlstP2,
            KzgPoly,
            KzgFFTSettings4844,
            KzgKZGSettings4844,
        >(
            &load_trusted_setup_rust,
            &blob_to_kzg_commitment_rust,
            &compute_aggregate_kzg_proof_rust,
            &verify_aggregate_kzg_proof_rust,
        );
    }
}
