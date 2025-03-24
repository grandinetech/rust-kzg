use super::utils::{get_manifest_dir, get_trusted_setup_path};
use crate::test_vectors::{
    compute_cells, compute_cells_and_kzg_proofs, recover_cells_and_kzg_proofs,
    verify_cell_kzg_proof_batch,
};
use kzg::{
    eth::{self, FIELD_ELEMENTS_PER_CELL},
    EcBackend, Fr, DAS, G1,
};
use std::{fs, path::PathBuf};

const COMPUTE_CELLS_AND_KZG_PROOFS_TEST_VECTORS: &str =
    "src/test_vectors/compute_cells_and_kzg_proofs/*/*/*";
const COMPUTE_CELLS_TEST_VECTORS: &str = "src/test_vectors/compute_cells/*/*/*";
const RECOVER_CELLS_AND_KZG_PROOFS_TEST_VECTORS: &str =
    "src/test_vectors/recover_cells_and_kzg_proofs/*/*/*";
const VERIFY_CELL_KZG_PROOF_BATCH_TEST_VECTORS: &str =
    "src/test_vectors/verify_cell_kzg_proof_batch/*/*/*";

#[allow(clippy::type_complexity)]
pub fn test_vectors_compute_cells_and_kzg_proofs<B: EcBackend>(
    load_trusted_setup: &dyn Fn(&str) -> Result<B::KZGSettings, String>,
    bytes_to_blob: &dyn Fn(&[u8]) -> Result<Vec<B::Fr>, String>,
) {
    let settings = load_trusted_setup(get_trusted_setup_path().as_str()).unwrap();
    let test_files: Vec<PathBuf> = glob::glob(&format!(
        "{}/{}",
        get_manifest_dir(),
        COMPUTE_CELLS_AND_KZG_PROOFS_TEST_VECTORS
    ))
    .unwrap()
    .collect::<Result<Vec<_>, _>>()
    .unwrap();
    assert!(!test_files.is_empty());

    for test_file in test_files {
        let yaml_data = fs::read_to_string(test_file.clone()).unwrap();
        let test: compute_cells_and_kzg_proofs::Test = serde_yaml::from_str(&yaml_data).unwrap();

        let blob = match bytes_to_blob(&test.input.get_blob_bytes().unwrap()) {
            Ok(blob) => blob,
            Err(_) => {
                assert!(test.get_output().is_none());
                continue;
            }
        };

        let mut recv_cells =
            vec![B::Fr::default(); eth::CELLS_PER_EXT_BLOB * eth::FIELD_ELEMENTS_PER_CELL];
        let mut recv_proofs = vec![B::G1::default(); eth::CELLS_PER_EXT_BLOB];

        match <B::KZGSettings as DAS<B>>::compute_cells_and_kzg_proofs(
            &settings,
            Some(&mut recv_cells),
            Some(&mut recv_proofs),
            &blob,
        ) {
            Err(_) => assert!(test.get_output().is_none()),
            Ok(()) => {
                let (exp_cells, exp_proofs) = test.get_output().unwrap();

                let recv_cells = recv_cells
                    .chunks(FIELD_ELEMENTS_PER_CELL)
                    .map(|it| it.iter().flat_map(|it| it.to_bytes()).collect::<Vec<_>>())
                    .collect::<Vec<Vec<u8>>>();
                let recv_proofs = recv_proofs
                    .into_iter()
                    .map(|it| it.to_bytes().to_vec())
                    .collect::<Vec<Vec<u8>>>();

                assert!(
                    recv_cells == exp_cells,
                    "Cells do not match, for test vector {:?}",
                    test_file
                );
                assert!(
                    recv_proofs == exp_proofs,
                    "Proofs do not match, for test vector {:?}",
                    test_file
                );
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn test_vectors_compute_cells<B: EcBackend>(
    load_trusted_setup: &dyn Fn(&str) -> Result<B::KZGSettings, String>,
    bytes_to_blob: &dyn Fn(&[u8]) -> Result<Vec<B::Fr>, String>,
) {
    let settings = load_trusted_setup(get_trusted_setup_path().as_str()).unwrap();
    let test_files: Vec<PathBuf> = glob::glob(&format!(
        "{}/{}",
        get_manifest_dir(),
        COMPUTE_CELLS_TEST_VECTORS
    ))
    .unwrap()
    .collect::<Result<Vec<_>, _>>()
    .unwrap();
    assert!(!test_files.is_empty());

    for test_file in test_files {
        let yaml_data = fs::read_to_string(test_file.clone()).unwrap();
        let test: compute_cells::Test = serde_yaml::from_str(&yaml_data).unwrap();

        let blob = match bytes_to_blob(&test.input.get_blob_bytes().unwrap()) {
            Ok(blob) => blob,
            Err(_) => {
                assert!(test.get_output().is_none());
                continue;
            }
        };

        let mut recv_cells =
            vec![B::Fr::default(); eth::CELLS_PER_EXT_BLOB * eth::FIELD_ELEMENTS_PER_CELL];

        match <B::KZGSettings as DAS<B>>::compute_cells_and_kzg_proofs(
            &settings,
            Some(&mut recv_cells),
            None,
            &blob,
        ) {
            Err(_) => assert!(test.get_output().is_none()),
            Ok(()) => {
                let exp_cells = test.get_output().unwrap();

                let recv_cells = recv_cells
                    .chunks(FIELD_ELEMENTS_PER_CELL)
                    .map(|it| it.iter().flat_map(|it| it.to_bytes()).collect::<Vec<_>>())
                    .collect::<Vec<Vec<u8>>>();
                assert!(
                    recv_cells == exp_cells,
                    "Cells do not match, for test vector {:?}",
                    test_file
                );
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn test_vectors_recover_cells_and_kzg_proofs<B: EcBackend>(
    load_trusted_setup: &dyn Fn(&str) -> Result<B::KZGSettings, String>,
) {
    let settings = load_trusted_setup(get_trusted_setup_path().as_str()).unwrap();
    let test_files: Vec<PathBuf> = glob::glob(&format!(
        "{}/{}",
        get_manifest_dir(),
        RECOVER_CELLS_AND_KZG_PROOFS_TEST_VECTORS
    ))
    .unwrap()
    .collect::<Result<Vec<_>, _>>()
    .unwrap();
    assert!(!test_files.is_empty());

    for test_file in test_files {
        let yaml_data = fs::read_to_string(test_file.clone()).unwrap();
        let test: recover_cells_and_kzg_proofs::Test = serde_yaml::from_str(&yaml_data).unwrap();

        let cells = match test
            .input
            .get_cell_bytes()
            .unwrap()
            .iter()
            .flat_map(|bytes| {
                bytes
                    .chunks(eth::BYTES_PER_FIELD_ELEMENT)
                    .map(B::Fr::from_bytes)
            })
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(v) => v,
            Err(err) => {
                // c-kzg-4844 also includes tests with invalid byte count for cell
                // in rust-kzg, these checks are performed outside of recovery function,
                // while parsing data (in c bindings, for example). These tests will be
                // additionally checked through rust-kzg c binding tests.

                // We add here assertion, to avoid accidentally skipping valid test case
                assert!(
                    test.get_output().is_none(),
                    "Parsing input failed with error {err:?}, for test vector {test_file:?}",
                );

                continue;
            }
        };

        let mut recv_cells =
            vec![B::Fr::default(); eth::CELLS_PER_EXT_BLOB * eth::FIELD_ELEMENTS_PER_CELL];

        let mut recv_proofs = vec![B::G1::default(); eth::CELLS_PER_EXT_BLOB];

        match <B::KZGSettings as DAS<B>>::recover_cells_and_kzg_proofs(
            &settings,
            &mut recv_cells,
            Some(&mut recv_proofs),
            &test.input.get_cell_indices().unwrap().iter().map(|it| (*it).into()).collect::<Vec<_>>(),
            &cells,
        ) {
            Err(err) => assert!(test.get_output().is_none(), "Should correctly recover cells, but failed with error {err:?}, for test vector {test_file:?}"),
            Ok(()) => {
                let test_output = test.get_output();

                assert!(test_output.is_some(), "Should fail, but succeeded for test vector {test_file:?}");

                let (exp_cells, exp_proofs) = test_output.unwrap();

                let recv_cells = recv_cells
                    .chunks(eth::FIELD_ELEMENTS_PER_CELL)
                    .map(|it| it.iter().flat_map(|it| it.to_bytes()).collect::<Vec<_>>())
                    .collect::<Vec<Vec<u8>>>();

                let recv_proofs = recv_proofs
                    .into_iter()
                    .map(|it| it.to_bytes().to_vec())
                    .collect::<Vec<Vec<u8>>>();

                assert!(
                    recv_cells == exp_cells,
                    "Cells do not match, for test vector {test_file:?}",
                );
                assert!(
                    recv_proofs == exp_proofs,
                    "Proofs do not match, for test vector {:?}",
                    test_file
                );
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn test_vectors_verify_cell_kzg_proof_batch<B: EcBackend>(
    load_trusted_setup: &dyn Fn(&str) -> Result<B::KZGSettings, String>,
) {
    let settings = load_trusted_setup(get_trusted_setup_path().as_str()).unwrap();
    let test_files: Vec<PathBuf> = glob::glob(&format!(
        "{}/{}",
        get_manifest_dir(),
        VERIFY_CELL_KZG_PROOF_BATCH_TEST_VECTORS
    ))
    .unwrap()
    .collect::<Result<Vec<_>, _>>()
    .unwrap();
    assert!(!test_files.is_empty());

    for test_file in test_files {
        let yaml_data = fs::read_to_string(test_file.clone()).unwrap();
        let test: verify_cell_kzg_proof_batch::Test = serde_yaml::from_str(&yaml_data).unwrap();

        let cells = match test
            .input
            .get_cell_bytes()
            .unwrap()
            .iter()
            .flat_map(|bytes| {
                bytes
                    .chunks(eth::BYTES_PER_FIELD_ELEMENT)
                    .map(B::Fr::from_bytes)
            })
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(v) => v,
            Err(err) => {
                // c-kzg-4844 also includes tests with invalid byte count for cell
                // in rust-kzg, these checks are performed outside of recovery function,
                // while parsing data (in c bindings, for example). These tests will be
                // additionally checked through rust-kzg c binding tests.

                // We add here assertion, to avoid accidentally skipping valid test case
                assert!(
                    test.get_output().is_none(),
                    "Parsing input failed with error {err:?}, for test vector {test_file:?}",
                );

                continue;
            }
        };

        let commitments = match test
            .input
            .get_commitment_bytes()
            .unwrap()
            .iter()
            .map(|bytes| B::G1::from_bytes(bytes))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(v) => v,
            Err(err) => {
                // c-kzg-4844 also includes tests with invalid byte count for cell
                // in rust-kzg, these checks are performed outside of recovery function,
                // while parsing data (in c bindings, for example). These tests will be
                // additionally checked through rust-kzg c binding tests.

                // We add here assertion, to avoid accidentally skipping valid test case
                assert!(
                    test.get_output().is_none(),
                    "Parsing input failed with error {err:?}, for test vector {test_file:?}",
                );

                continue;
            }
        };

        let proofs = match test
            .input
            .get_proof_bytes()
            .unwrap()
            .iter()
            .map(|bytes| B::G1::from_bytes(bytes))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(v) => v,
            Err(err) => {
                // c-kzg-4844 also includes tests with invalid byte count for cell
                // in rust-kzg, these checks are performed outside of recovery function,
                // while parsing data (in c bindings, for example). These tests will be
                // additionally checked through rust-kzg c binding tests.

                // We add here assertion, to avoid accidentally skipping valid test case
                assert!(
                    test.get_output().is_none(),
                    "Parsing input failed with error {err:?}, for test vector {test_file:?}",
                );

                continue;
            }
        };

        let cell_indices = test.input.get_cell_indices().unwrap();

        match <B::KZGSettings as DAS<B>>::verify_cell_kzg_proof_batch(
            &settings,
            &commitments,
            &cell_indices,
            &cells,
            &proofs,
        ) {
            Err(err) => assert!(test.get_output().is_none(), "Should correctly verify cells, but failed with error {err:?}, for test vector {test_file:?}"),
            Ok(value) => {
                let test_output = test.get_output();

                assert!(test_output.is_some(), "Should fail, but succeeded for test vector {test_file:?}");
                assert_eq!(value, test_output.unwrap(), "Test vector failed {test_file:?}");
            }
        }
    }
}
