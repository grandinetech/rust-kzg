#[cfg(test)]

mod tests {

    use ckzg::{
        consts::{BlstP1, BlstP2},
        eip_4844::{
            bytes_from_bls_field, bound_bytes_to_bls_field,
            compute_powers, 
            load_trusted_setup_rust, bytes_from_g1_rust, verify_aggregate_kzg_proof_rust, compute_aggregate_kzg_proof_rust, blob_to_kzg_commitment_rust,
        },
        finite::BlstFr,
        poly::KzgPoly, kzgsettings4844::KzgKZGSettings4844, fftsettings4844::KzgFFTSettings4844,
    };
    use kzg_bench::tests::eip_4844::{
        bytes_to_bls_field_test, compute_powers_test,
        eip4844_test, blob_to_kzg_commitment_test, compute_aggregate_kzg_proof_test_empty, aggregate_proof_for_single_blob_test,
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
    pub fn eip4844_test_(){
        eip4844_test::<BlstFr, BlstP1, BlstP2, KzgPoly, KzgFFTSettings4844, KzgKZGSettings4844>(&load_trusted_setup_rust, &blob_to_kzg_commitment_rust, &compute_aggregate_kzg_proof_rust, &verify_aggregate_kzg_proof_rust);
    }

    #[test]
    pub fn blob_to_kzg_commitment_test_(){
        blob_to_kzg_commitment_test::<BlstFr, BlstP1, BlstP2, KzgPoly, KzgFFTSettings4844, KzgKZGSettings4844>(&load_trusted_setup_rust, &blob_to_kzg_commitment_rust, &bytes_from_g1_rust);
    }

    #[test]
    pub fn compute_aggregate_kzg_proof_test_empty_() {
        compute_aggregate_kzg_proof_test_empty::<BlstFr, BlstP1, BlstP2, KzgPoly, KzgFFTSettings4844, KzgKZGSettings4844>(
            &load_trusted_setup_rust,
            &compute_aggregate_kzg_proof_rust,
            &bytes_from_g1_rust,
        )
    }

    #[test]
    pub fn aggregate_proof_for_single_blob_test_() {
        aggregate_proof_for_single_blob_test::<BlstFr, BlstP1, BlstP2, KzgPoly, KzgFFTSettings4844, KzgKZGSettings4844>(
            &load_trusted_setup_rust,
            &blob_to_kzg_commitment_rust,
            &compute_aggregate_kzg_proof_rust,
            &verify_aggregate_kzg_proof_rust,
        );
    }

}
