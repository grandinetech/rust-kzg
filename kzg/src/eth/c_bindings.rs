use blst::{blst_fr, blst_p1, blst_p1_affine, blst_p2};

use crate::{
    eth::{CELLS_PER_EXT_BLOB, FIELD_ELEMENTS_PER_CELL},
    EcBackend, Fr, DAS, G1,
};

use super::{
    Mainnet, BYTES_PER_BLOB, BYTES_PER_CELL, BYTES_PER_COMMITMENT, BYTES_PER_FIELD_ELEMENT,
    BYTES_PER_PROOF,
};

use crate::alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

#[repr(C)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CKzgRet {
    Ok = 0,
    BadArgs = 1,
    Error = 2,
    Malloc = 3,
}

#[repr(C)]
pub struct Bytes32 {
    pub bytes: [u8; 32],
}

#[repr(C)]
pub struct Bytes48 {
    pub bytes: [u8; 48],
}

#[repr(C)]
pub struct BLSFieldElement {
    pub bytes: [u8; BYTES_PER_FIELD_ELEMENT],
}

#[repr(C)]
pub struct Blob {
    pub bytes: [u8; BYTES_PER_BLOB],
}

#[repr(C)]
pub struct KZGCommitment {
    pub bytes: [u8; BYTES_PER_COMMITMENT],
}

#[repr(C)]
pub struct KZGProof {
    pub bytes: [u8; BYTES_PER_PROOF],
}

#[repr(C)]
pub struct CKZGSettings {
    /**
     * Roots of unity for the subgroup of size `FIELD_ELEMENTS_PER_EXT_BLOB`.
     *
     * The array contains `FIELD_ELEMENTS_PER_EXT_BLOB + 1` elements.
     * The array starts and ends with Fr::one().
     */
    pub roots_of_unity: *mut blst_fr,
    /**
     * Roots of unity for the subgroup of size `FIELD_ELEMENTS_PER_EXT_BLOB` in bit-reversed order.
     *
     * This array is derived by applying a bit-reversal permutation to `roots_of_unity`
     * excluding the last element. Essentially:
     *   `brp_roots_of_unity = bit_reversal_permutation(roots_of_unity[:-1])`
     *
     * The array contains `FIELD_ELEMENTS_PER_EXT_BLOB` elements.
     */
    pub brp_roots_of_unity: *mut blst_fr,
    /**
     * Roots of unity for the subgroup of size `FIELD_ELEMENTS_PER_EXT_BLOB` in reversed order.
     *
     * It is the reversed version of `roots_of_unity`. Essentially:
     *    `reverse_roots_of_unity = reverse(roots_of_unity)`
     *
     * This array is primarily used in FFTs.
     * The array contains `FIELD_ELEMENTS_PER_EXT_BLOB + 1` elements.
     * The array starts and ends with Fr::one().
     */
    pub reverse_roots_of_unity: *mut blst_fr,
    /**
     * G1 group elements from the trusted setup in monomial form.
     * The array contains `NUM_G1_POINTS = FIELD_ELEMENTS_PER_BLOB` elements.
     */
    pub g1_values_monomial: *mut blst_p1,
    /**
     * G1 group elements from the trusted setup in Lagrange form and bit-reversed order.
     * The array contains `NUM_G1_POINTS = FIELD_ELEMENTS_PER_BLOB` elements.
     */
    pub g1_values_lagrange_brp: *mut blst_p1,
    /**
     * G2 group elements from the trusted setup in monomial form.
     * The array contains `NUM_G2_POINTS` elements.
     */
    pub g2_values_monomial: *mut blst_p2,
    /** Data used during FK20 proof generation. */
    pub x_ext_fft_columns: *mut *mut blst_p1,
    /** The precomputed tables for fixed-base MSM. */
    pub tables: *mut *mut blst_p1_affine,
    /** The window size for the fixed-base MSM. */
    pub wbits: usize,
    /** The scratch size for the fixed-base MSM. */
    pub scratch_size: usize,
}

#[repr(C)]
pub struct Cell {
    pub bytes: [u8; BYTES_PER_CELL],
}

unsafe fn deserialize_blob<B: EcBackend>(
    blob: *const Blob,
) -> core::result::Result<Vec<B::Fr>, CKzgRet> {
    (*blob)
        .bytes
        .chunks(BYTES_PER_FIELD_ELEMENT)
        .map(|chunk| {
            let mut bytes = [0u8; BYTES_PER_FIELD_ELEMENT];
            bytes.copy_from_slice(chunk);
            if let Ok(result) = B::Fr::from_bytes(&bytes) {
                Ok(result)
            } else {
                Err(CKzgRet::BadArgs)
            }
        })
        .collect::<Result<Vec<_>, _>>()
}

/// # Safety
pub unsafe fn compute_cells_and_kzg_proofs<
    B: EcBackend,
    D: DAS<B, FIELD_ELEMENTS_PER_CELL, Mainnet> + for<'a> TryFrom<&'a CKZGSettings, Error = String>,
>(
    cells: *mut Cell,
    proofs: *mut KZGProof,
    blob: *const Blob,
    settings: *const CKZGSettings,
) -> CKzgRet {
    unsafe fn inner<
        B: EcBackend,
        D: DAS<B, FIELD_ELEMENTS_PER_CELL, Mainnet>
            + for<'a> TryFrom<&'a CKZGSettings, Error = String>,
    >(
        cells: *mut Cell,
        proofs: *mut KZGProof,
        blob: *const Blob,
        settings: *const CKZGSettings,
    ) -> Result<(), String> {
        let mut cells_rs: Option<Vec<[B::Fr; FIELD_ELEMENTS_PER_CELL]>> = if cells.is_null() {
            None
        } else {
            Some(vec![
                core::array::from_fn(|_| B::Fr::default());
                CELLS_PER_EXT_BLOB
            ])
        };
        let mut proofs_rs = if proofs.is_null() {
            None
        } else {
            Some(vec![B::G1::default(); CELLS_PER_EXT_BLOB])
        };

        let blob = deserialize_blob::<B>(blob).map_err(|_| "Invalid blob".to_string())?;
        let settings: D = (&*settings).try_into()?;

        settings.compute_cells_and_kzg_proofs(
            cells_rs.as_deref_mut(),
            proofs_rs.as_deref_mut(),
            &blob,
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

    match inner::<B, D>(cells, proofs, blob, settings) {
        Ok(()) => CKzgRet::Ok,
        Err(_) => CKzgRet::BadArgs,
    }
}

/// # Safety
pub unsafe fn recover_cells_and_kzg_proofs<
    B: EcBackend,
    D: DAS<B, FIELD_ELEMENTS_PER_CELL, Mainnet> + for<'a> TryFrom<&'a CKZGSettings, Error = String>,
>(
    recovered_cells: *mut Cell,
    recovered_proofs: *mut KZGProof,
    cell_indices: *const u64,
    cells: *const Cell,
    num_cells: u64,
    s: *const CKZGSettings,
) -> CKzgRet {
    unsafe fn inner<
        B: EcBackend,
        D: DAS<B, FIELD_ELEMENTS_PER_CELL, Mainnet>
            + for<'a> TryFrom<&'a CKZGSettings, Error = String>,
    >(
        recovered_cells: *mut Cell,
        recovered_proofs: *mut KZGProof,
        cell_indices: *const u64,
        cells: *const Cell,
        num_cells: u64,
        s: *const CKZGSettings,
    ) -> Result<(), String> {
        let mut recovered_cells_rs: Vec<[B::Fr; FIELD_ELEMENTS_PER_CELL]> =
            vec![core::array::from_fn(|_| B::Fr::default()); CELLS_PER_EXT_BLOB];

        let mut recovered_proofs_rs = if recovered_proofs.is_null() {
            None
        } else {
            Some(vec![B::G1::default(); CELLS_PER_EXT_BLOB])
        };

        let cell_indicies = core::slice::from_raw_parts(cell_indices, num_cells as usize)
            .iter()
            .map(|it| *it as usize)
            .collect::<Vec<_>>();
        let cells = core::slice::from_raw_parts(cells, num_cells as usize)
            .iter()
            .map(|it| -> Result<[B::Fr; FIELD_ELEMENTS_PER_CELL], String> {
                it.bytes
                    .chunks(BYTES_PER_FIELD_ELEMENT)
                    .map(B::Fr::from_bytes)
                    .collect::<Result<Vec<_>, String>>()
                    .and_then(|frs| {
                        frs.try_into()
                            .map_err(|_| "Invalid field element count per cell".to_string())
                    })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let settings: D = (&*s).try_into()?;

        settings.recover_cells_and_kzg_proofs(
            &mut recovered_cells_rs,
            recovered_proofs_rs.as_deref_mut(),
            &cell_indicies,
            &cells,
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

    match inner::<B, D>(
        recovered_cells,
        recovered_proofs,
        cell_indices,
        cells,
        num_cells,
        s,
    ) {
        Ok(()) => CKzgRet::Ok,
        Err(_) => CKzgRet::BadArgs,
    }
}

/// # Safety
pub unsafe fn verify_cell_kzg_proof_batch<
    B: EcBackend,
    D: DAS<B, FIELD_ELEMENTS_PER_CELL, Mainnet> + for<'a> TryFrom<&'a CKZGSettings, Error = String>,
>(
    ok: *mut bool,
    commitments_bytes: *const Bytes48,
    cell_indices: *const u64,
    cells: *const Cell,
    proofs_bytes: *const Bytes48,
    num_cells: u64,
    s: *const CKZGSettings,
) -> CKzgRet {
    unsafe fn inner<
        B: EcBackend,
        D: DAS<B, FIELD_ELEMENTS_PER_CELL, Mainnet>
            + for<'a> TryFrom<&'a CKZGSettings, Error = String>,
    >(
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
            .map(|bytes| B::G1::from_bytes(&bytes.bytes))
            .collect::<Result<Vec<_>, String>>()?;

        let cell_indices = core::slice::from_raw_parts(cell_indices, num_cells as usize)
            .iter()
            .map(|it| *it as usize)
            .collect::<Vec<_>>();

        let cells = core::slice::from_raw_parts(cells, num_cells as usize)
            .iter()
            .map(|it| -> Result<[B::Fr; FIELD_ELEMENTS_PER_CELL], String> {
                it.bytes
                    .chunks(BYTES_PER_FIELD_ELEMENT)
                    .map(B::Fr::from_bytes)
                    .collect::<Result<Vec<_>, String>>()
                    .and_then(|frs| {
                        frs.try_into()
                            .map_err(|_| "Invalid field element count per cell".to_string())
                    })
            })
            .collect::<Result<Vec<_>, String>>()?;

        let proofs = core::slice::from_raw_parts(proofs_bytes, num_cells as usize)
            .iter()
            .map(|bytes| B::G1::from_bytes(&bytes.bytes))
            .collect::<Result<Vec<_>, String>>()?;

        let settings: D = (&*s).try_into()?;

        *ok = settings.verify_cell_kzg_proof_batch(&commitments, &cell_indices, &cells, &proofs)?;

        Ok(())
    }

    match inner::<B, D>(
        ok,
        commitments_bytes,
        cell_indices,
        cells,
        proofs_bytes,
        num_cells,
        s,
    ) {
        Ok(()) => CKzgRet::Ok,
        Err(_) => CKzgRet::BadArgs,
    }
}

#[macro_export]
macro_rules! c_bindings_eip7594 {
    ($backend:ty) => {
        /// # Safety
        #[no_mangle]
        pub unsafe extern "C" fn compute_cells_and_kzg_proofs(
            cells: *mut kzg::eth::c_bindings::Cell,
            proofs: *mut kzg::eth::c_bindings::KZGProof,
            blob: *const kzg::eth::c_bindings::Blob,
            settings: *const kzg::eth::c_bindings::CKZGSettings,
        ) -> kzg::eth::c_bindings::CKzgRet {
            kzg::eth::c_bindings::compute_cells_and_kzg_proofs::<
                $backend,
                <$backend as kzg::EcBackend>::KZGSettings,
            >(cells, proofs, blob, settings)
        }

        /// # Safety
        #[no_mangle]
        pub unsafe extern "C" fn recover_cells_and_kzg_proofs(
            recovered_cells: *mut kzg::eth::c_bindings::Cell,
            recovered_proofs: *mut kzg::eth::c_bindings::KZGProof,
            cell_indices: *const u64,
            cells: *const kzg::eth::c_bindings::Cell,
            num_cells: u64,
            s: *const kzg::eth::c_bindings::CKZGSettings,
        ) -> kzg::eth::c_bindings::CKzgRet {
            kzg::eth::c_bindings::recover_cells_and_kzg_proofs::<
                $backend,
                <$backend as kzg::EcBackend>::KZGSettings,
            >(
                recovered_cells,
                recovered_proofs,
                cell_indices,
                cells,
                num_cells,
                s,
            )
        }

        /// # Safety
        #[no_mangle]
        pub unsafe extern "C" fn verify_cell_kzg_proof_batch(
            ok: *mut bool,
            commitments_bytes: *const kzg::eth::c_bindings::Bytes48,
            cell_indices: *const u64,
            cells: *const kzg::eth::c_bindings::Cell,
            proofs_bytes: *const kzg::eth::c_bindings::Bytes48,
            num_cells: u64,
            s: *const kzg::eth::c_bindings::CKZGSettings,
        ) -> kzg::eth::c_bindings::CKzgRet {
            kzg::eth::c_bindings::verify_cell_kzg_proof_batch::<
                $backend,
                <$backend as kzg::EcBackend>::KZGSettings,
            >(
                ok,
                commitments_bytes,
                cell_indices,
                cells,
                proofs_bytes,
                num_cells,
                s,
            )
        }
    };
}
