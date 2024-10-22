use kzg::{eip_4844::FIELD_ELEMENTS_PER_CELL};

use crate::{kzg_types::{ZFr, ZG1}, kzg_proofs::KZGSettings};

extern crate alloc;

// pub fn recover_cells_and_kzg_proofs_rust(
//     recovered_cells: &mut [[FsFr; FIELD_ELEMENTS_PER_CELL]],
//     recovered_proofs: Option<&mut [FsG1]>,
//     cell_indicies: &[usize],
//     cells: &[[FsFr; FIELD_ELEMENTS_PER_CELL]],
//     s: &FsKZGSettings,
// ) -> Result<(), String> {
//     kzg::eip_7594::recover_cells_and_kzg_proofs(
//         recovered_cells,
//         recovered_proofs,
//         cell_indicies,
//         cells,
//         s,
//     )
// }

pub fn compute_cells_and_kzg_proofs_rust(
    cells: Option<&mut [[ZFr; FIELD_ELEMENTS_PER_CELL]]>,
    proofs: Option<&mut [ZG1]>,
    blob: &[ZFr],
    s: &KZGSettings
) -> Result<(), String> {
    kzg::eip_7594::compute_cells_and_kzg_proofs(cells, proofs, blob, s)
}

pub fn recover_cells_and_kzg_proofs_rust(
    recovered_cells: &mut [[ZFr; FIELD_ELEMENTS_PER_CELL]],
    recovered_proofs: Option<&mut [ZG1]>,
    cell_indicies: &[usize],
    cells: &[[ZFr; FIELD_ELEMENTS_PER_CELL]],
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
    commitments: &[ZG1],
    cell_indices: &[usize],
    cells: &[[ZFr; FIELD_ELEMENTS_PER_CELL]],
    proofs: &[ZG1],
    s: &KZGSettings,
) -> Result<bool, String> {
    kzg::eip_7594::verify_cell_kzg_proof_batch(commitments, cell_indices, cells, proofs, s)
}