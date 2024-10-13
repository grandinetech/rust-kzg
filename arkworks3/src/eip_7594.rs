use kzg::eip_4844::FIELD_ELEMENTS_PER_CELL;

use crate::kzg_types::{ArkFr, ArkG1, ArkKZGSettings};

pub fn compute_cells_and_kzg_proofs_rust(
    cells: Option<&mut [[ArkFr; FIELD_ELEMENTS_PER_CELL]]>,
    proofs: Option<&mut [ArkG1]>,
    blob: &[ArkFr],
    s: &ArkKZGSettings,
) -> Result<(), String> {
    kzg::eip_7594::compute_cells_and_kzg_proofs(cells, proofs, blob, s)
}

pub fn recover_cells_and_kzg_proofs_rust(
    recovered_cells: &mut [[ArkFr; FIELD_ELEMENTS_PER_CELL]],
    recovered_proofs: Option<&mut [ArkG1]>,
    cell_indicies: &[usize],
    cells: &[[ArkFr; FIELD_ELEMENTS_PER_CELL]],
    s: &ArkKZGSettings,
) -> Result<(), String> {
    kzg::eip_7594::recover_cells_and_kzg_proofs(
        recovered_cells,
        recovered_proofs,
        cell_indicies,
        cells,
        s,
    )
}

pub fn verify_cell_kzg_proof_batch_rust(
    commitments: &[ArkG1],
    cell_indices: &[usize],
    cells: &[[ArkFr; FIELD_ELEMENTS_PER_CELL]],
    proofs: &[ArkG1],
    s: &ArkKZGSettings,
) -> Result<bool, String> {
    kzg::eip_7594::verify_cell_kzg_proof_batch(commitments, cell_indices, cells, proofs, s)
}