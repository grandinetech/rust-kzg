use super::utils::{get_manifest_dir, get_trusted_setup_path};
use crate::test_vectors::compute_cells_and_kzg_proofs;
use kzg::{FFTSettings, Fr, G1Affine, G1Fp, G1GetFp, G1Mul, KZGSettings, Poly, G1, G2};
use std::path::PathBuf;

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
        &[TFr],
        &TKZGSettings,
    ) -> Result<(Vec<TFr>, Vec<TG1>), String>,
    bytes_to_blob: &dyn Fn(&[u8]) -> Result<Vec<TFr>, String>,
) {
    let settings = load_trusted_setup(get_trusted_setup_path().as_str()).unwrap();
    let test_files: Vec<PathBuf> = glob::glob(&format!(
        "{}/{}",
        get_manifest_dir(),
        COMPUTE_CELLS_AND_KZG_PROOFS_TEST_VECTORS
    ))
    .unwrap()
    .collect::<Vec<Result<_>>>()
    .unwrap();
    assert!(!test_files.is_empty());

    for test_file in test_files {
        let yaml_data = fs::read_to_string(test_file).unwrap();
        let test: compute_cells_and_kzg_proofs::Test = serde_yaml::from_str(&yaml_data).unwrap();

        let blob = match bytes_to_blob(&test.input.get_blob_bytes()) {
            Ok(blob) => blob,
            Err(_) => {
                assert!(test.get_output().is_none());
                continue;
            }
        };

        match compute_cells_and_kzg_proofs(&blob, &settings) {
            Err(_) => assert!(test.get_output().is_none()),
            Ok((recv_cells, recv_proofs)) => {
                let (exp_cells, exp_proofs) = test.get_output().unwrap();

                let recv_cells = recv_cells
                    .into_iter()
                    .map(|it| it.to_bytes().to_vec())
                    .collect::<Vec<Vec<u8>>>();
                let recv_proofs = recv_proofs
                    .into_iter()
                    .map(|it| it.to_bytes().to_vec())
                    .collect::<Vec<Vec<u8>>>();

                assert_eq!(recv_cells, exp_cells, "Cells do not match");
                assert_eq!(recv_proofs, exp_proofs, "Proofs do not match");
            }
        }
    }
}
