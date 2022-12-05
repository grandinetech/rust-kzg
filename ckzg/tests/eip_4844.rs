#[cfg(test)]

mod tests {

    use ckzg::{
        consts::{BlstP1, BlstP2},
        eip_4844::{
            blob_to_kzg_commitment_rust, bound_bytes_to_bls_field, bytes_from_bls_field,
            bytes_from_g1_rust, compute_aggregate_kzg_proof_rust, compute_powers,
            load_trusted_setup_rust, verify_aggregate_kzg_proof_rust,
        },
        fftsettings4844::KzgFFTSettings4844,
        finite::BlstFr,
        kzgsettings4844::KzgKZGSettings4844,
        poly::KzgPoly,
    };
    use kzg_bench::tests::eip_4844::{
        aggregate_proof_for_single_blob_test, blob_to_kzg_commitment_test, bytes_to_bls_field_test,
        compute_aggregate_kzg_proof_test_empty, compute_powers_test, eip4844_test,
    };

    #[test]
    pub fn bytes_to_bls_field_test_() {
        bytes_to_bls_field_test::<BlstFr>(&bound_bytes_to_bls_field, &bytes_from_bls_field);
    }

    #[test]
    pub fn compute_powers_test_() {
        compute_powers_test::<BlstFr>(&bound_bytes_to_bls_field, &compute_powers);
    }

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
    pub fn blob_to_kzg_commitment_test_() {
        blob_to_kzg_commitment_test::<
            BlstFr,
            BlstP1,
            BlstP2,
            KzgPoly,
            KzgFFTSettings4844,
            KzgKZGSettings4844,
        >(
            &load_trusted_setup_rust,
            &blob_to_kzg_commitment_rust,
            &bytes_from_g1_rust,
        );
    }

    #[test]
    pub fn compute_aggregate_kzg_proof_test_empty_() {
        compute_aggregate_kzg_proof_test_empty::<
            BlstFr,
            BlstP1,
            BlstP2,
            KzgPoly,
            KzgFFTSettings4844,
            KzgKZGSettings4844,
        >(
            &load_trusted_setup_rust,
            &compute_aggregate_kzg_proof_rust,
            &bytes_from_g1_rust,
        )
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
