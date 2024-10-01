use super::utils::{get_manifest_dir, get_trusted_setup_path};
use crate::test_vectors::{
    compute_cells_and_kzg_proofs, recover_cells_and_kzg_proofs, verify_cell_kzg_proof_batch,
};
use kzg::{
    eip_4844::{BYTES_PER_FIELD_ELEMENT, CELLS_PER_EXT_BLOB, FIELD_ELEMENTS_PER_CELL},
    FFTSettings, Fr, G1Affine, G1Fp, G1GetFp, G1Mul, KZGSettings, Poly, G1, G2,
};
use std::{fmt::Debug, fs, path::PathBuf};

const COMPUTE_CELLS_AND_KZG_PROOFS_TEST_VECTORS: &str =
    "src/test_vectors/compute_cells_and_kzg_proofs/*/*/*";
const RECOVER_CELLS_AND_KZG_PROOFS_TEST_VECTORS: &str =
    "src/test_vectors/recover_cells_and_kzg_proofs/*/*/*";
const VERIFY_CELL_KZG_PROOF_BATCH_TEST_VECTORS: &str =
    "src/test_vectors/verify_cell_kzg_proof_batch/*/*/*";

pub fn test_vectors_compute_cells_and_kzg_proofs<
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TG1Fp, TG1Affine>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
>(
    load_trusted_setup: &dyn Fn(&str) -> Result<TKZGSettings, String>,
    compute_cells_and_kzg_proofs: &dyn Fn(
        Option<&mut [[TFr; FIELD_ELEMENTS_PER_CELL]]>,
        Option<&mut [TG1]>,
        &[TFr],
        &TKZGSettings,
    ) -> Result<(), String>,
    bytes_to_blob: &dyn Fn(&[u8]) -> Result<Vec<TFr>, String>,
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
            vec![
                core::array::from_fn::<_, FIELD_ELEMENTS_PER_CELL, _>(|_| TFr::default());
                CELLS_PER_EXT_BLOB
            ];
        let mut recv_proofs = vec![TG1::default(); CELLS_PER_EXT_BLOB];

        match compute_cells_and_kzg_proofs(
            Some(&mut recv_cells),
            Some(&mut recv_proofs),
            &blob,
            &settings,
        ) {
            Err(_) => assert!(test.get_output().is_none()),
            Ok(()) => {
                let (exp_cells, exp_proofs) = test.get_output().unwrap();

                let recv_cells = recv_cells
                    .into_iter()
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
                assert_eq!(
                    recv_proofs, exp_proofs,
                    "Proofs do not match, for test vector {:?}",
                    test_file
                );
            }
        }
    }
}

pub fn test_vectors_recover_cells_and_kzg_proofs<
    TFr: Fr + Debug,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TG1Fp, TG1Affine>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
>(
    load_trusted_setup: &dyn Fn(&str) -> Result<TKZGSettings, String>,
    recover_cells_and_kzg_proofs: &dyn Fn(
        &mut [[TFr; FIELD_ELEMENTS_PER_CELL]],
        Option<&mut [TG1]>,
        &[usize],
        &[[TFr; FIELD_ELEMENTS_PER_CELL]],
        &TKZGSettings,
    ) -> Result<(), String>,
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
            .map(|bytes| {
                match bytes
                    .chunks(BYTES_PER_FIELD_ELEMENT)
                    .map(|bytes| TFr::from_bytes(bytes))
                    .collect::<Result<Vec<_>, String>>()
                {
                    Ok(value) => value
                        .try_into()
                        .map_err(|_| "Invalid field element per cell count".to_string()),
                    Err(err) => Err(err),
                }
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

        let mut recv_cells = vec![
            vec![TFr::default(); FIELD_ELEMENTS_PER_CELL]
                .try_into()
                .unwrap();
            CELLS_PER_EXT_BLOB
        ];

        let mut recv_proofs = vec![TG1::default(); CELLS_PER_EXT_BLOB];

        match recover_cells_and_kzg_proofs(
            &mut recv_cells,
            Some(&mut recv_proofs),
            &test.input.get_cell_indices().unwrap().iter().map(|it| (*it).into()).collect::<Vec<_>>(),
            &cells,
            &settings,
        ) {
            Err(err) => assert!(test.get_output().is_none(), "Should correctly recover cells, but failed with error {err:?}, for test vector {test_file:?}"),
            Ok(()) => {
                let test_output = test.get_output();

                assert!(test_output.is_some(), "Should fail, but succeeded for test vector {test_file:?}");

                let (exp_cells, exp_proofs) = test_output.unwrap();

                let recv_cells = recv_cells
                    .into_iter()
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
                assert_eq!(
                    recv_proofs, exp_proofs,
                    "Proofs do not match, for test vector {:?}",
                    test_file
                );
            }
        }
    }
}

pub fn test_vectors_verify_cell_kzg_proof_batch<
    TFr: Fr + Debug,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TG1Fp, TG1Affine>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
>(
    load_trusted_setup: &dyn Fn(&str) -> Result<TKZGSettings, String>,
    verify_cell_kzg_proof_batch: &dyn Fn(
        &[TG1],
        &[usize],
        &[[TFr; FIELD_ELEMENTS_PER_CELL]],
        &[TG1],
        &TKZGSettings,
    ) -> Result<bool, String>,
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
        if test_file.parent().unwrap().file_name().unwrap().to_str().unwrap() != "recover_cells_and_kzg_proofs_case_valid_half_missing_every_other_cell_0d06acb410563a7d" {
            continue;
        }

        let yaml_data = fs::read_to_string(test_file.clone()).unwrap();
        let test: verify_cell_kzg_proof_batch::Test = serde_yaml::from_str(&yaml_data).unwrap();

        let cells = match test
            .input
            .get_cell_bytes()
            .unwrap()
            .iter()
            .map(|bytes| {
                match bytes
                    .chunks(BYTES_PER_FIELD_ELEMENT)
                    .map(|bytes| TFr::from_bytes(bytes))
                    .collect::<Result<Vec<_>, String>>()
                {
                    Ok(value) => value
                        .try_into()
                        .map_err(|_| "Invalid field element per cell count".to_string()),
                    Err(err) => Err(err),
                }
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
            .map(|bytes| TG1::from_bytes(&bytes))
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
            .map(|bytes| TG1::from_bytes(&bytes))
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

        match verify_cell_kzg_proof_batch(
            &commitments,
            &cell_indices,
            &cells,
            &proofs,
            &settings,
        ) {
            Err(err) => assert!(test.get_output().is_none(), "Should correctly recover cells, but failed with error {err:?}, for test vector {test_file:?}"),
            Ok(value) => {
                let test_output = test.get_output();

                assert!(test_output.is_some(), "Should fail, but succeeded for test vector {test_file:?}");
                assert_eq!(value, test_output.unwrap(), "Test vector failed {test_file:?}");
            }
        }
    }
}
