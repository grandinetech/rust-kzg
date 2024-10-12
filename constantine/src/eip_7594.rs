use kzg::eip_4844::FIELD_ELEMENTS_PER_CELL;

use crate::types::{fr::CtFr, g1::CtG1, kzg_settings::CtKZGSettings};

pub fn compute_cells_and_kzg_proofs_rust(
    cells: Option<&mut [[CtFr; FIELD_ELEMENTS_PER_CELL]]>,
    proofs: Option<&mut [CtG1]>,
    blob: &[CtFr],
    s: &CtKZGSettings,
) -> Result<(), String> {
    kzg::eip_7594::compute_cells_and_kzg_proofs(cells, proofs, blob, s)
}

pub fn recover_cells_and_kzg_proofs_rust(
    recovered_cells: &mut [[CtFr; FIELD_ELEMENTS_PER_CELL]],
    recovered_proofs: Option<&mut [CtG1]>,
    cell_indicies: &[usize],
    cells: &[[CtFr; FIELD_ELEMENTS_PER_CELL]],
    s: &CtKZGSettings,
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
    commitments: &[CtG1],
    cell_indices: &[usize],
    cells: &[[CtFr; FIELD_ELEMENTS_PER_CELL]],
    proofs: &[CtG1],
    s: &CtKZGSettings,
) -> Result<bool, String> {
    kzg::eip_7594::verify_cell_kzg_proof_batch(commitments, cell_indices, cells, proofs, s)
}
