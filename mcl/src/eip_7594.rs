use kzg::eip_4844::FIELD_ELEMENTS_PER_CELL;

use crate::{
    kzg_settings::KZGSettings,
    data_types::{fr::Fr, g1::G1},
};

extern crate alloc;

pub fn compute_cells_and_kzg_proofs_rust(
    cells: Option<&mut [[Fr; FIELD_ELEMENTS_PER_CELL]]>,
    proofs: Option<&mut [G1]>,
    blob: &[Fr],
    s: &KZGSettings,
) -> Result<(), String> {
    kzg::eip_7594::compute_cells_and_kzg_proofs(cells, proofs, blob, s)
}

pub fn recover_cells_and_kzg_proofs_rust(
    recovered_cells: &mut [[Fr; FIELD_ELEMENTS_PER_CELL]],
    recovered_proofs: Option<&mut [G1]>,
    cell_indicies: &[usize],
    cells: &[[Fr; FIELD_ELEMENTS_PER_CELL]],
    s: &KZGSettings,
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
    commitments: &[G1],
    cell_indices: &[usize],
    cells: &[[Fr; FIELD_ELEMENTS_PER_CELL]],
    proofs: &[G1],
    s: &KZGSettings,
) -> Result<bool, String> {
    kzg::eip_7594::verify_cell_kzg_proof_batch(commitments, cell_indices, cells, proofs, s)
}
