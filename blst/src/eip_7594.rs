extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::{
    types::g1::FsG1,
    utils::{deserialize_blob, kzg_settings_to_rust},
};
use kzg::{
    eip_4844::{
        Blob, Bytes48, CKZGSettings, Cell, KZGProof, BYTES_PER_FIELD_ELEMENT, CELLS_PER_EXT_BLOB,
        C_KZG_RET, C_KZG_RET_BADARGS, C_KZG_RET_OK, FIELD_ELEMENTS_PER_CELL,
    },
    Fr, G1,
};

use crate::types::{fr::FsFr, kzg_settings::FsKZGSettings};

pub fn compute_cells_and_kzg_proofs_rust(
    cells: Option<&mut [[FsFr; FIELD_ELEMENTS_PER_CELL]]>,
    proofs: Option<&mut [FsG1]>,
    blob: &[FsFr],
    s: &FsKZGSettings,
) -> Result<(), String> {
    kzg::eip_7594::compute_cells_and_kzg_proofs(cells, proofs, blob, s)
}

pub fn recover_cells_and_kzg_proofs_rust(
    recovered_cells: &mut [[FsFr; FIELD_ELEMENTS_PER_CELL]],
    recovered_proofs: Option<&mut [FsG1]>,
    cell_indicies: &[usize],
    cells: &[[FsFr; FIELD_ELEMENTS_PER_CELL]],
    s: &FsKZGSettings,
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
    commitments: &[FsG1],
    cell_indices: &[usize],
    cells: &[[FsFr; FIELD_ELEMENTS_PER_CELL]],
    proofs: &[FsG1],
    s: &FsKZGSettings,
) -> Result<bool, String> {
    kzg::eip_7594::verify_cell_kzg_proof_batch(commitments, cell_indices, cells, proofs, s)
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn compute_cells_and_kzg_proofs(
    cells: *mut Cell,
    proofs: *mut KZGProof,
    blob: *const Blob,
    settings: *const CKZGSettings,
) -> C_KZG_RET {
    unsafe fn inner(
        cells: *mut Cell,
        proofs: *mut KZGProof,
        blob: *const Blob,
        settings: *const CKZGSettings,
    ) -> Result<(), String> {
        let mut cells_rs = if cells.is_null() {
            None
        } else {
            Some(vec![
                [FsFr::default(); FIELD_ELEMENTS_PER_CELL];
                CELLS_PER_EXT_BLOB
            ])
        };
        let mut proofs_rs = if proofs.is_null() {
            None
        } else {
            Some(vec![FsG1::default(); CELLS_PER_EXT_BLOB])
        };

        let blob = deserialize_blob(blob).map_err(|_| "Invalid blob".to_string())?;
        let settings = kzg_settings_to_rust(&*settings)?;

        compute_cells_and_kzg_proofs_rust(
            cells_rs.as_deref_mut(),
            proofs_rs.as_deref_mut(),
            &blob,
            &settings,
        )?;

        if let Some(cells_rs) = cells_rs {
            let cells = core::slice::from_raw_parts_mut(cells, CELLS_PER_EXT_BLOB);
            for (cell_index, cell) in cells_rs.iter().enumerate() {
                for (fr_index, fr) in cell.iter().enumerate() {
                    cells[cell_index].bytes[(fr_index * BYTES_PER_FIELD_ELEMENT)
                        ..((fr_index + 1) * BYTES_PER_FIELD_ELEMENT)]
                        .copy_from_slice(&fr.to_bytes());
                }
            }
        }

        if let Some(proofs_rs) = proofs_rs {
            let proofs = core::slice::from_raw_parts_mut(proofs, CELLS_PER_EXT_BLOB);
            for (proof_index, proof) in proofs_rs.iter().enumerate() {
                proofs[proof_index].bytes.copy_from_slice(&proof.to_bytes());
            }
        }

        Ok(())
    }

    match inner(cells, proofs, blob, settings) {
        Ok(()) => C_KZG_RET_OK,
        Err(_) => C_KZG_RET_BADARGS,
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn recover_cells_and_kzg_proofs(
    recovered_cells: *mut Cell,
    recovered_proofs: *mut KZGProof,
    cell_indices: *const u64,
    cells: *const Cell,
    num_cells: u64,
    s: *const CKZGSettings,
) -> C_KZG_RET {
    unsafe fn inner(
        recovered_cells: *mut Cell,
        recovered_proofs: *mut KZGProof,
        cell_indices: *const u64,
        cells: *const Cell,
        num_cells: u64,
        s: *const CKZGSettings,
    ) -> Result<(), String> {
        let mut recovered_cells_rs =
            vec![[FsFr::default(); FIELD_ELEMENTS_PER_CELL]; CELLS_PER_EXT_BLOB];

        let mut recovered_proofs_rs = if recovered_proofs.is_null() {
            None
        } else {
            Some(vec![FsG1::default(); CELLS_PER_EXT_BLOB])
        };

        let cell_indicies = core::slice::from_raw_parts(cell_indices, num_cells as usize)
            .iter()
            .map(|it| *it as usize)
            .collect::<Vec<_>>();
        let cells = core::slice::from_raw_parts(cells, num_cells as usize)
            .iter()
            .map(|it| -> Result<[FsFr; FIELD_ELEMENTS_PER_CELL], String> {
                it.bytes
                    .chunks(BYTES_PER_FIELD_ELEMENT)
                    .map(FsFr::from_bytes)
                    .collect::<Result<Vec<_>, String>>()
                    .and_then(|frs| {
                        frs.try_into()
                            .map_err(|_| "Invalid field element count per cell".to_string())
                    })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let settings = kzg_settings_to_rust(&*s)?;

        recover_cells_and_kzg_proofs_rust(
            &mut recovered_cells_rs,
            recovered_proofs_rs.as_deref_mut(),
            &cell_indicies,
            &cells,
            &settings,
        )?;

        let recovered_cells = core::slice::from_raw_parts_mut(recovered_cells, CELLS_PER_EXT_BLOB);
        for (cell_c, cell_rs) in recovered_cells.iter_mut().zip(recovered_cells_rs.iter()) {
            cell_c.bytes.copy_from_slice(
                &cell_rs
                    .iter()
                    .flat_map(|fr| fr.to_bytes())
                    .collect::<Vec<_>>(),
            );
        }

        if let Some(recovered_proofs_rs) = recovered_proofs_rs {
            let recovered_proofs =
                core::slice::from_raw_parts_mut(recovered_proofs, CELLS_PER_EXT_BLOB);

            for (proof_c, proof_rs) in recovered_proofs.iter_mut().zip(recovered_proofs_rs.iter()) {
                proof_c.bytes = proof_rs.to_bytes();
            }
        }

        Ok(())
    }

    match inner(
        recovered_cells,
        recovered_proofs,
        cell_indices,
        cells,
        num_cells,
        s,
    ) {
        Ok(()) => C_KZG_RET_OK,
        Err(_) => C_KZG_RET_BADARGS,
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn verify_cell_kzg_proof_batch(
    ok: *mut bool,
    commitments_bytes: *const Bytes48,
    cell_indices: *const u64,
    cells: *const Cell,
    proofs_bytes: *const Bytes48,
    num_cells: u64,
    s: *const CKZGSettings,
) -> C_KZG_RET {
    unsafe fn inner(
        ok: *mut bool,
        commitments_bytes: *const Bytes48,
        cell_indices: *const u64,
        cells: *const Cell,
        proofs_bytes: *const Bytes48,
        num_cells: u64,
        s: *const CKZGSettings,
    ) -> Result<(), String> {
        let commitments = core::slice::from_raw_parts(commitments_bytes, num_cells as usize)
            .iter()
            .map(|bytes| FsG1::from_bytes(&bytes.bytes))
            .collect::<Result<Vec<_>, String>>()?;

        let cell_indices = core::slice::from_raw_parts(cell_indices, num_cells as usize)
            .iter()
            .map(|it| *it as usize)
            .collect::<Vec<_>>();

        let cells = core::slice::from_raw_parts(cells, num_cells as usize)
            .iter()
            .map(|it| -> Result<[FsFr; FIELD_ELEMENTS_PER_CELL], String> {
                it.bytes
                    .chunks(BYTES_PER_FIELD_ELEMENT)
                    .map(FsFr::from_bytes)
                    .collect::<Result<Vec<_>, String>>()
                    .and_then(|frs| {
                        frs.try_into()
                            .map_err(|_| "Invalid field element count per cell".to_string())
                    })
            })
            .collect::<Result<Vec<_>, String>>()?;

        let proofs = core::slice::from_raw_parts(proofs_bytes, num_cells as usize)
            .iter()
            .map(|bytes| FsG1::from_bytes(&bytes.bytes))
            .collect::<Result<Vec<_>, String>>()?;

        let settings = kzg_settings_to_rust(&*s)?;

        *ok = verify_cell_kzg_proof_batch_rust(
            &commitments,
            &cell_indices,
            &cells,
            &proofs,
            &settings,
        )?;

        Ok(())
    }

    match inner(
        ok,
        commitments_bytes,
        cell_indices,
        cells,
        proofs_bytes,
        num_cells,
        s,
    ) {
        Ok(()) => C_KZG_RET_OK,
        Err(_) => C_KZG_RET_BADARGS,
    }
}
