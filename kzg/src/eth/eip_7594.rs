#[cfg(feature = "parallel")]
use rayon::prelude::*;

use alloc::{format, string::String, vec::Vec};

use crate::{
    cfg_chunks, cfg_iter,
    das::{EcBackend, DAS},
    eip_4844::bytes_to_blob,
    eth::{
        BYTES_PER_BLOB, BYTES_PER_CELL, BYTES_PER_COMMITMENT, BYTES_PER_FIELD_ELEMENT,
        BYTES_PER_PROOF, CELLS_PER_EXT_BLOB, FIELD_ELEMENTS_PER_CELL, FIELD_ELEMENTS_PER_EXT_BLOB,
    },
    Fr, G1,
};

pub type CellsKzgProofs = (Vec<[u8; BYTES_PER_CELL]>, Vec<[u8; BYTES_PER_PROOF]>);

pub fn recover_cells_and_kzg_proofs_raw<B: EcBackend>(
    cell_indices: &[usize],
    cells: &[[u8; BYTES_PER_CELL]],
    das: &impl DAS<B>,
) -> Result<CellsKzgProofs, String>
where
    B::G1: Copy,
    B::Fr: Copy,
{
    let cells = cfg_chunks!(cells.as_flattened(), BYTES_PER_FIELD_ELEMENT)
        .map(B::Fr::from_bytes)
        .collect::<Result<Vec<_>, _>>()?;

    let mut recovered_cells = [B::Fr::default(); FIELD_ELEMENTS_PER_EXT_BLOB];
    let mut recovered_proofs = [B::G1::default(); CELLS_PER_EXT_BLOB];

    das.recover_cells_and_kzg_proofs(
        &mut recovered_cells,
        Some(&mut recovered_proofs),
        cell_indices,
        &cells,
    )?;

    let converted_cells = cells_elements_to_cells_bytes::<B>(&recovered_cells)?;
    let converted_proofs = recovered_proofs
        .into_iter()
        .map(|proof| proof.to_bytes())
        .collect::<Vec<_>>();

    Ok((converted_cells, converted_proofs))
}

pub fn compute_cells_and_kzg_proofs_raw<B: EcBackend>(
    blob: [u8; BYTES_PER_BLOB],
    das: &impl DAS<B>,
) -> Result<CellsKzgProofs, String>
where
    B::G1: Copy,
    B::Fr: Copy,
{
    let blob = bytes_to_blob(&blob)?;

    let mut recovered_cells = [B::Fr::default(); FIELD_ELEMENTS_PER_EXT_BLOB];
    let mut recovered_proofs = [B::G1::default(); CELLS_PER_EXT_BLOB];

    das.compute_cells_and_kzg_proofs(
        Some(&mut recovered_cells),
        Some(&mut recovered_proofs),
        &blob,
    )?;

    let converted_cells = cells_elements_to_cells_bytes::<B>(&recovered_cells)?;
    let converted_proofs = recovered_proofs
        .into_iter()
        .map(|proof| proof.to_bytes())
        .collect::<Vec<_>>();

    Ok((converted_cells, converted_proofs))
}

pub fn compute_cells_raw<B: EcBackend>(
    blob: [u8; BYTES_PER_BLOB],
    das: &impl DAS<B>,
) -> Result<Vec<[u8; BYTES_PER_CELL]>, String>
where
    B::Fr: Copy,
{
    let blob = bytes_to_blob(&blob)?;
    let mut recovered_cells = [B::Fr::default(); FIELD_ELEMENTS_PER_EXT_BLOB];

    das.compute_cells_and_kzg_proofs(Some(&mut recovered_cells), None, &blob)?;

    let converted_cells = cells_elements_to_cells_bytes::<B>(&recovered_cells)?;

    Ok(converted_cells)
}

pub fn verify_cell_kzg_proof_batch_raw<B: EcBackend>(
    commitments: &[[u8; BYTES_PER_COMMITMENT]],
    cell_indices: &[usize],
    cells: &[[u8; BYTES_PER_CELL]],
    proofs: &[[u8; BYTES_PER_PROOF]],
    das: &impl DAS<B>,
) -> Result<bool, String> {
    let commitments = cfg_iter!(commitments)
        .map(|commitment| B::G1::from_bytes(commitment))
        .collect::<Result<Vec<_>, _>>()?;

    let cells = cfg_chunks!(cells.as_flattened(), BYTES_PER_FIELD_ELEMENT)
        .map(B::Fr::from_bytes)
        .collect::<Result<Vec<_>, _>>()?;

    let proofs = cfg_iter!(proofs)
        .map(|proof| B::G1::from_bytes(proof))
        .collect::<Result<Vec<_>, _>>()?;

    das.verify_cell_kzg_proof_batch(&commitments, cell_indices, &cells, &proofs)
}

fn cells_elements_to_cells_bytes<B: EcBackend>(
    bytes: &[B::Fr],
) -> Result<Vec<[u8; BYTES_PER_CELL]>, String> {
    // NOTE: chunk_size = BYTES_PER_CELL / BYTES_PER_FIELD_ELEMENT
    if bytes.len() != FIELD_ELEMENTS_PER_EXT_BLOB {
        return Err(format!(
            "Invalid field elements length. Expected {} got {}",
            FIELD_ELEMENTS_PER_EXT_BLOB,
            bytes.len(),
        ));
    }

    Ok(cfg_chunks!(bytes, FIELD_ELEMENTS_PER_CELL)
        .map(|cell_bytes| {
            let mut result = [0u8; BYTES_PER_CELL];
            for (idx, field_element) in cell_bytes.iter().enumerate() {
                let bytes_element = field_element.to_bytes();
                let start = idx * BYTES_PER_FIELD_ELEMENT;
                let end = start + BYTES_PER_FIELD_ELEMENT;
                result[start..end].copy_from_slice(&bytes_element);
            }
            result
        })
        .collect())
}
