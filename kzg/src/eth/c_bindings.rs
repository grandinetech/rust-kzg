use crate::{
    eth::{CELLS_PER_EXT_BLOB, FIELD_ELEMENTS_PER_CELL},
    EcBackend, Fr, DAS, G1,
};

use super::{
    BYTES_PER_BLOB, BYTES_PER_CELL, BYTES_PER_COMMITMENT, BYTES_PER_FIELD_ELEMENT, BYTES_PER_PROOF,
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
    D: DAS<B> + for<'a> TryFrom<&'a CKZGSettings, Error = String>,
>(
    cells: *mut Cell,
    proofs: *mut KZGProof,
    blob: *const Blob,
    settings: *const CKZGSettings,
) -> CKzgRet {
    unsafe fn inner<B: EcBackend, D: DAS<B> + for<'a> TryFrom<&'a CKZGSettings, Error = String>>(
        cells: *mut Cell,
        proofs: *mut KZGProof,
        blob: *const Blob,
        settings: *const CKZGSettings,
    ) -> Result<(), String> {
        let mut cells_rs: Option<Vec<B::Fr>> = if cells.is_null() {
            None
        } else {
            Some(vec![
                B::Fr::default();
                CELLS_PER_EXT_BLOB * FIELD_ELEMENTS_PER_CELL
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

            for (cell_rs, cell_c) in cells_rs.chunks(FIELD_ELEMENTS_PER_CELL).zip(cells) {
                cell_c.bytes.copy_from_slice(
                    &cell_rs
                        .iter()
                        .flat_map(|fr| fr.to_bytes())
                        .collect::<Vec<u8>>(),
                );
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
    D: DAS<B> + for<'a> TryFrom<&'a CKZGSettings, Error = String>,
>(
    recovered_cells: *mut Cell,
    recovered_proofs: *mut KZGProof,
    cell_indices: *const u64,
    cells: *const Cell,
    num_cells: u64,
    s: *const CKZGSettings,
) -> CKzgRet {
    unsafe fn inner<B: EcBackend, D: DAS<B> + for<'a> TryFrom<&'a CKZGSettings, Error = String>>(
        recovered_cells: *mut Cell,
        recovered_proofs: *mut KZGProof,
        cell_indices: *const u64,
        cells: *const Cell,
        num_cells: u64,
        s: *const CKZGSettings,
    ) -> Result<(), String> {
        let mut recovered_cells_rs: Vec<B::Fr> =
            vec![B::Fr::default(); FIELD_ELEMENTS_PER_CELL * CELLS_PER_EXT_BLOB];

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
            .flat_map(|it| {
                it.bytes
                    .chunks(BYTES_PER_FIELD_ELEMENT)
                    .map(B::Fr::from_bytes)
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
        for (cell_c, cell_rs) in recovered_cells
            .iter_mut()
            .zip(recovered_cells_rs.chunks(FIELD_ELEMENTS_PER_CELL))
        {
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
    D: DAS<B> + for<'a> TryFrom<&'a CKZGSettings, Error = String>,
>(
    ok: *mut bool,
    commitments_bytes: *const Bytes48,
    cell_indices: *const u64,
    cells: *const Cell,
    proofs_bytes: *const Bytes48,
    num_cells: u64,
    s: *const CKZGSettings,
) -> CKzgRet {
    unsafe fn inner<B: EcBackend, D: DAS<B> + for<'a> TryFrom<&'a CKZGSettings, Error = String>>(
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
            .flat_map(|it| {
                it.bytes
                    .chunks(BYTES_PER_FIELD_ELEMENT)
                    .map(B::Fr::from_bytes)
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

#[macro_export]
macro_rules! c_bindings_rust_eth_kzg {
    ($backend:ty, $path:expr) => {
        pub struct DASContext {
            pub inner: <$backend as kzg::EcBackend>::KZGSettings,
        }

        /// A C-style enum to indicate whether a function call was a success or not.
        #[repr(C)]
        pub enum CResultStatus {
            Ok,
            Err,
        }

        /// A C-style struct to represent the success result of a function call.
        ///
        /// This includes the status of the call and an error message, if the status was an error.
        #[repr(C)]
        pub struct CResult {
            pub status: CResultStatus,
            pub error_msg: *mut std::os::raw::c_char,
        }

        impl CResult {
            /// Create a new CResult with an error message.
            ///
            /// # Memory leaks
            ///
            /// - Ownership of the error message is transferred to the caller.
            ///   The caller is responsible for freeing the memory allocated for the error message.
            ///   This can be done by calling `eth_kzg_free_error_message`.
            ///
            /// # Memory faults
            ///
            /// - If this method is called twice on the same pointer, it will result in a double-free.
            pub fn with_error(error_msg: &str) -> Self {
                let error_msg = std::ffi::CString::new(error_msg).unwrap();
                CResult {
                    status: CResultStatus::Err,
                    error_msg: error_msg.into_raw(),
                }
            }

            /// Creates a new CResult with an Ok status indicating a function has returned successfully.
            pub fn with_ok() -> Self {
                CResult {
                    status: CResultStatus::Ok,
                    error_msg: std::ptr::null_mut(),
                }
            }
        }

        /// # Safety
        #[no_mangle]
        pub extern "C" fn eth_kzg_das_context_new(
            _use_precomp: bool,
            _num_threads: u32,
        ) -> *mut DASContext {
            let (g1_monomial_bytes, g1_lagrange_bytes, g2_monomial_bytes) =
                kzg::eip_4844::load_trusted_setup_string(include_str!($path)).unwrap();

            let ctx = Box::new(DASContext {
                inner: kzg::eip_4844::load_trusted_setup_rust::<
                    <$backend as kzg::EcBackend>::Fr,
                    <$backend as kzg::EcBackend>::G1,
                    <$backend as kzg::EcBackend>::G2,
                    <$backend as kzg::EcBackend>::FFTSettings,
                    <$backend as kzg::EcBackend>::Poly,
                    <$backend as kzg::EcBackend>::KZGSettings,
                    <$backend as kzg::EcBackend>::G1Fp,
                    <$backend as kzg::EcBackend>::G1Affine,
                >(&g1_monomial_bytes, &g1_lagrange_bytes, &g2_monomial_bytes)
                .unwrap(),
            });

            Box::into_raw(ctx)
        }

        /// # Safety
        #[no_mangle]
        pub extern "C" fn eth_kzg_das_context_free(ctx: *mut DASContext) {
            if ctx.is_null() {
                return;
            }

            unsafe {
                let _ = Box::from_raw(ctx);
            }
        }

        /// Free the memory allocated for the error message.
        ///
        /// # Safety
        ///
        /// - The caller must ensure that the pointer is valid. If the pointer is null, this method will return early.
        /// - The caller should also avoid a double-free by setting the pointer to null after calling this method.
        #[no_mangle]
        pub unsafe extern "C" fn eth_kzg_free_error_message(c_message: *mut std::os::raw::c_char) {
            // check if the pointer is null
            if c_message.is_null() {
                return;
            }
            // Safety: Deallocate the memory allocated for the C-style string
            unsafe {
                let _ = std::ffi::CString::from_raw(c_message);
            };
        }

        /// # Safety
        #[no_mangle]
        #[must_use]
        pub extern "C" fn eth_kzg_blob_to_kzg_commitment(
            ctx: *const DASContext,
            blob: *const u8,
            out: *mut u8,
        ) -> CResult {
            use kzg::G1;

            let ctx = unsafe { &*ctx };
            let blob = unsafe { core::slice::from_raw_parts(blob, kzg::eth::BYTES_PER_BLOB) };
            let out = unsafe { core::slice::from_raw_parts_mut(out, kzg::eth::BYTES_PER_G1) };

            match kzg::eip_4844::blob_to_kzg_commitment_raw(blob.try_into().unwrap(), &ctx.inner) {
                Ok(result) => {
                    out.copy_from_slice(&result.to_bytes());

                    CResult::with_ok()
                }
                Err(err) => CResult::with_error(err.as_str()),
            }
        }

        /// # Safety
        #[no_mangle]
        #[must_use]
        pub extern "C" fn eth_kzg_compute_cells_and_kzg_proofs(
            ctx: *const DASContext,
            blob: *const u8,
            out_cells: *mut *mut u8,
            out_proofs: *mut *mut u8,
        ) -> CResult {
            let ctx = unsafe { &*ctx };

            let blob = unsafe { core::slice::from_raw_parts(blob, kzg::eth::BYTES_PER_BLOB) };
            let out_cells =
                unsafe { core::slice::from_raw_parts_mut(out_cells, kzg::eth::CELLS_PER_EXT_BLOB) };
            let out_proofs = unsafe {
                core::slice::from_raw_parts_mut(out_proofs, kzg::eth::CELLS_PER_EXT_BLOB)
            };

            match kzg::eth::eip_7594::compute_cells_and_kzg_proofs_raw::<$backend>(
                blob.try_into().unwrap(),
                &ctx.inner,
            ) {
                Ok((cells, proofs)) => {
                    for (cell, &out_cell) in cells.into_iter().zip(out_cells.iter()) {
                        let out_cell = unsafe {
                            core::slice::from_raw_parts_mut(out_cell, kzg::eth::BYTES_PER_CELL)
                        };

                        out_cell.copy_from_slice(&cell);
                    }

                    for (proof, &out_proof) in proofs.into_iter().zip(out_proofs.iter()) {
                        let out_proof = unsafe {
                            core::slice::from_raw_parts_mut(
                                out_proof,
                                kzg::eth::BYTES_PER_COMMITMENT,
                            )
                        };

                        out_proof.copy_from_slice(&proof);
                    }

                    CResult::with_ok()
                }
                Err(err) => CResult::with_error(err.as_str()),
            }
        }

        /// # Safety
        #[no_mangle]
        #[must_use]
        pub extern "C" fn eth_kzg_verify_cell_kzg_proof_batch(
            ctx: *const DASContext,

            commitments_length: u64,
            commitments: *const *const u8,

            cell_indices_length: u64,
            cell_indices: *const u64,

            cells_length: u64,
            cells: *const *const u8,

            proofs_length: u64,
            proofs: *const *const u8,

            verified: *mut bool,
        ) -> CResult {
            let ctx = unsafe { &(*ctx) };

            let commitments =
                unsafe { core::slice::from_raw_parts(commitments, commitments_length as usize) };
            let commitments = commitments
                .into_iter()
                .map(|&c| {
                    unsafe { core::slice::from_raw_parts(c, kzg::eth::BYTES_PER_COMMITMENT) }
                        .try_into()
                        .unwrap()
                })
                .collect::<Vec<_>>();

            let cell_indices =
                unsafe { core::slice::from_raw_parts(cell_indices, cell_indices_length as usize) };
            let cell_indices = cell_indices
                .into_iter()
                .map(|&i| i as usize)
                .collect::<Vec<_>>();

            let cells = unsafe { core::slice::from_raw_parts(cells, cells_length as usize) };
            let cells = cells
                .into_iter()
                .map(|&c| {
                    unsafe { core::slice::from_raw_parts(c, kzg::eth::BYTES_PER_CELL) }
                        .try_into()
                        .unwrap()
                })
                .collect::<Vec<_>>();

            let proofs = unsafe { core::slice::from_raw_parts(proofs, proofs_length as usize) };
            let proofs = proofs
                .into_iter()
                .map(|&p| {
                    unsafe { core::slice::from_raw_parts(p, kzg::eth::BYTES_PER_COMMITMENT) }
                        .try_into()
                        .unwrap()
                })
                .collect::<Vec<_>>();

            let verified = unsafe { &mut *verified };

            match kzg::eth::eip_7594::verify_cell_kzg_proof_batch_raw::<$backend>(
                &commitments,
                &cell_indices,
                &cells,
                &proofs,
                &ctx.inner,
            ) {
                Ok(result) => {
                    *verified = result;

                    CResult::with_ok()
                }
                Err(err) => CResult::with_error(err.as_str()),
            }
        }

        /// # Safety
        #[no_mangle]
        #[must_use]
        pub extern "C" fn eth_kzg_recover_cells_and_proofs(
            ctx: *const DASContext,

            cells_length: u64,
            cells: *const *const u8,

            cell_indices_length: u64,
            cell_indices: *const u64,

            out_cells: *mut *mut u8,
            out_proofs: *mut *mut u8,
        ) -> CResult {
            let ctx = unsafe { &(*ctx) };

            let cells = unsafe { core::slice::from_raw_parts(cells, cells_length as usize) };
            let cells = cells
                .into_iter()
                .map(|&c| {
                    unsafe { core::slice::from_raw_parts(c, kzg::eth::BYTES_PER_CELL) }
                        .try_into()
                        .unwrap()
                })
                .collect::<Vec<_>>();

            let cell_indices =
                unsafe { core::slice::from_raw_parts(cell_indices, cell_indices_length as usize) };
            let cell_indices = cell_indices
                .into_iter()
                .map(|&i| i as usize)
                .collect::<Vec<_>>();

            let out_cells =
                unsafe { core::slice::from_raw_parts_mut(out_cells, kzg::eth::CELLS_PER_EXT_BLOB) };
            let out_proofs = unsafe {
                core::slice::from_raw_parts_mut(out_proofs, kzg::eth::CELLS_PER_EXT_BLOB)
            };

            match kzg::eth::eip_7594::recover_cells_and_kzg_proofs_raw::<$backend>(
                &cell_indices,
                &cells,
                &ctx.inner,
            ) {
                Ok((cells, proofs)) => {
                    for (cell, &out_cell) in cells.into_iter().zip(out_cells.iter()) {
                        let out_cell = unsafe {
                            core::slice::from_raw_parts_mut(out_cell, kzg::eth::BYTES_PER_CELL)
                        };
                        out_cell.copy_from_slice(&cell);
                    }

                    for (proof, &out_proof) in proofs.into_iter().zip(out_proofs.iter()) {
                        let out_proof = unsafe {
                            core::slice::from_raw_parts_mut(
                                out_proof,
                                kzg::eth::BYTES_PER_COMMITMENT,
                            )
                        };
                        out_proof.copy_from_slice(&proof);
                    }

                    CResult::with_ok()
                }
                Err(err) => CResult::with_error(err.as_str()),
            }
        }

        #[no_mangle]
        pub extern "C" fn eth_kzg_constant_bytes_per_cell() -> u64 {
            kzg::eth::BYTES_PER_CELL as u64
        }

        #[no_mangle]
        pub extern "C" fn eth_kzg_constant_bytes_per_proof() -> u64 {
            kzg::eth::BYTES_PER_COMMITMENT as u64
        }

        #[no_mangle]
        pub extern "C" fn eth_kzg_constant_cells_per_ext_blob() -> u64 {
            kzg::eth::CELLS_PER_EXT_BLOB as u64
        }
    };
}

#[macro_export]
macro_rules! c_bindings_java_eth_kzg {
    ($backend:ty) => {
        #[no_mangle]
        pub extern "system" fn Java_ethereum_cryptography_LibEthKZG_DASContextNew(
            _env: JNIEnv,
            _class: JClass,
            use_precomp: jboolean,
            num_threads: jlong,
        ) -> jlong {
            let use_precomp = use_precomp != 0;
            let num_threads = (num_threads as u64) as u32;
            eth_kzg_das_context_new(use_precomp, num_threads) as jlong
        }

        #[no_mangle]
        pub extern "system" fn Java_ethereum_cryptography_LibEthKZG_DASContextDestroy(
            _env: JNIEnv,
            _class: JClass,
            ctx_ptr: jlong,
        ) {
            eth_kzg_das_context_free(ctx_ptr as *mut DASContext);
        }

        #[no_mangle]
        pub extern "system" fn Java_ethereum_cryptography_LibEthKZG_computeCellsAndKZGProofs<
            'local,
        >(
            mut env: JNIEnv<'local>,
            _class: JClass,
            ctx_ptr: jlong,
            blob: JByteArray<'local>,
        ) -> JObject<'local> {
            let ctx = unsafe { &*(ctx_ptr as *const DASContext) };
            match compute_cells_and_kzg_proofs(&mut env, ctx, blob) {
                Ok(cells_and_proofs) => cells_and_proofs,
                Err(err) => {
                    throw_on_error(&mut env, err, "computeCellsAndKZGProofs");
                    JObject::default()
                }
            }
        }

        fn compute_cells_and_kzg_proofs<'local>(
            env: &mut JNIEnv<'local>,
            ctx: &DASContext,
            blob: JByteArray<'local>,
        ) -> Result<JObject<'local>, Error> {
            let blob = env.convert_byte_array(blob)?;
            let blob = slice_to_array_ref(&blob, "blob")?;

            let (cells, proofs) = kzg::eth::eip_7594::compute_cells_and_kzg_proofs_raw::<$backend>(
                *blob, &ctx.inner,
            )?;

            cells_and_proofs_to_jobject(env, &cells, &proofs).map_err(Error::from)
        }

        #[no_mangle]
        pub extern "system" fn Java_ethereum_cryptography_LibEthKZG_blobToKZGCommitment<'local>(
            mut env: JNIEnv<'local>,
            _class: JClass,
            ctx_ptr: jlong,
            blob: JByteArray<'local>,
        ) -> JByteArray<'local> {
            let ctx = unsafe { &*(ctx_ptr as *const DASContext) };
            match blob_to_kzg_commitment(&mut env, ctx, blob) {
                Ok(commitment) => commitment,
                Err(err) => {
                    throw_on_error(&mut env, err, "blobToKZGCommitment");
                    JByteArray::default()
                }
            }
        }

        fn blob_to_kzg_commitment<'local>(
            env: &mut JNIEnv<'local>,
            ctx: &DASContext,
            blob: JByteArray<'local>,
        ) -> Result<JByteArray<'local>, Error> {
            use kzg::G1;

            let blob = env.convert_byte_array(blob)?;
            let blob = slice_to_array_ref(&blob, "blob")?;

            let commitment = kzg::eip_4844::blob_to_kzg_commitment_raw(*blob, &ctx.inner)?;
            env.byte_array_from_slice(&commitment.to_bytes())
                .map_err(Error::from)
        }

        #[no_mangle]
        pub extern "system" fn Java_ethereum_cryptography_LibEthKZG_verifyCellKZGProofBatch<
            'local,
        >(
            mut env: JNIEnv<'local>,
            _class: JClass,
            ctx_ptr: jlong,
            commitment: JObjectArray<'local>,
            cell_indices: JLongArray,
            cells: JObjectArray<'local>,
            proofs: JObjectArray<'local>,
        ) -> jboolean {
            let ctx = unsafe { &*(ctx_ptr as *const DASContext) };

            match verify_cell_kzg_proof_batch(
                &mut env,
                ctx,
                commitment,
                cell_indices,
                cells,
                proofs,
            ) {
                Ok(result) => result,
                Err(err) => {
                    throw_on_error(&mut env, err, "verifyCellKZGProofBatch");
                    jboolean::default()
                }
            }
        }
        fn verify_cell_kzg_proof_batch<'local>(
            env: &mut JNIEnv,
            ctx: &DASContext,
            commitment: JObjectArray<'local>,
            cell_indices: JLongArray,
            cells: JObjectArray<'local>,
            proofs: JObjectArray<'local>,
        ) -> Result<jboolean, Error> {
            let commitment = jobject_array_to_2d_byte_array(env, commitment)?;
            let cell_indices = jlongarray_to_vec_u64(env, cell_indices)?;
            let cells = jobject_array_to_2d_byte_array(env, cells)?;
            let proofs = jobject_array_to_2d_byte_array(env, proofs)?;

            let cells: Vec<_> = cells
                .iter()
                .map(|cell| slice_to_array_ref(cell, "cell").map(|c| *c))
                .collect::<Result<_, _>>()?;
            let commitments: Vec<_> = commitment
                .iter()
                .map(|commitment| slice_to_array_ref(commitment, "commitment").map(|c| *c))
                .collect::<Result<_, _>>()?;
            let proofs: Vec<_> = proofs
                .iter()
                .map(|proof| slice_to_array_ref(proof, "proof").map(|c| *c))
                .collect::<Result<_, _>>()?;

            match kzg::eth::eip_7594::verify_cell_kzg_proof_batch_raw::<$backend>(
                &commitments,
                &cell_indices,
                &cells,
                &proofs,
                &ctx.inner,
            ) {
                Ok(res) => Ok(jboolean::from(res)),
                Err(err) => Err(Error::Cryptography(err)),
            }
        }

        #[no_mangle]
        pub extern "system" fn Java_ethereum_cryptography_LibEthKZG_recoverCellsAndKZGProofs<
            'local,
        >(
            mut env: JNIEnv<'local>,
            _class: JClass,
            ctx_ptr: jlong,
            cell_ids: JLongArray,
            cells: JObjectArray<'local>,
        ) -> JObject<'local> {
            let ctx = unsafe { &*(ctx_ptr as *const DASContext) };

            match recover_cells_and_kzg_proofs(&mut env, ctx, cell_ids, cells) {
                Ok(cells_and_proofs) => cells_and_proofs,
                Err(err) => {
                    throw_on_error(&mut env, err, "recoverCellsAndKZGProofs");
                    JObject::default()
                }
            }
        }
        fn recover_cells_and_kzg_proofs<'local>(
            env: &mut JNIEnv<'local>,
            ctx: &DASContext,
            cell_ids: JLongArray,
            cells: JObjectArray<'local>,
        ) -> Result<JObject<'local>, Error> {
            let cell_ids = jlongarray_to_vec_u64(env, cell_ids)?;
            let cells = jobject_array_to_2d_byte_array(env, cells)?;
            let cells: Vec<_> = cells
                .iter()
                .map(|cell| slice_to_array_ref(cell, "cell").map(|c| *c))
                .collect::<Result<_, _>>()?;

            let (recovered_cells, recovered_proofs) =
                kzg::eth::eip_7594::recover_cells_and_kzg_proofs_raw::<$backend>(
                    &cell_ids, &cells, &ctx.inner,
                )?;
            cells_and_proofs_to_jobject(env, &recovered_cells, &recovered_proofs)
                .map_err(Error::from)
        }

        /// Converts a JLongArray to a Vec<u64>
        fn jlongarray_to_vec_u64(env: &JNIEnv, array: JLongArray) -> Result<Vec<usize>, Error> {
            // Step 1: Get the length of the JLongArray
            let array_length = env.get_array_length(&array)?;

            // Step 2: Create a buffer to hold the jlong elements (these are i64s)
            let mut buffer: Vec<i64> = vec![0; array_length as usize];

            // Step 3: Get the elements from the JLongArray
            env.get_long_array_region(array, 0, &mut buffer)?;

            // Step 4: Convert the Vec<i64> to Vec<u64>
            Ok(buffer.into_iter().map(|x| x as usize).collect())
        }

        /// Converts a JObjectArray to a Vec<Vec<u8>>
        fn jobject_array_to_2d_byte_array(
            env: &mut JNIEnv,
            array: JObjectArray,
        ) -> Result<Vec<Vec<u8>>, Error> {
            // Get the length of the outer array
            let outer_len = env.get_array_length(&array)?;

            let mut result = Vec::with_capacity(outer_len as usize);

            for i in 0..outer_len {
                // Get each inner array (JByteArray)
                let inner_array_obj = env.get_object_array_element(&array, i)?;
                let inner_array: JByteArray = JByteArray::from(inner_array_obj);

                // Get the length of the inner array
                let inner_len = env.get_array_length(&inner_array)?;

                // Get the elements of the inner array
                let mut buf = vec![0; inner_len as usize];
                env.get_byte_array_region(inner_array, 0, &mut buf)?;

                // Convert i8 to u8
                let buf = buf.into_iter().map(|x| x as u8).collect();

                result.push(buf);
            }

            Ok(result)
        }

        /// Converts a Vec<Vec<u8>> to a JObject that represents a CellsAndProofs object in Java
        fn cells_and_proofs_to_jobject<'local>(
            env: &mut JNIEnv<'local>,
            cells: &[impl AsRef<[u8]>],
            proofs: &[impl AsRef<[u8]>],
        ) -> Result<JObject<'local>, Error> {
            // Create a new instance of the CellsAndProofs class in Java
            let cells_and_proofs_class = env.find_class("ethereum/cryptography/CellsAndProofs")?;

            let cell_byte_array_class = env.find_class("[B")?;
            let proof_byte_array_class = env.find_class("[B")?;

            // Create 2D array for cells
            let cells_array = env.new_object_array(
                cells.len() as i32,
                cell_byte_array_class,
                env.new_byte_array(0)?,
            )?;

            for (i, cell) in cells.iter().enumerate() {
                let cell_array = env.byte_array_from_slice(cell.as_ref())?;
                env.set_object_array_element(&cells_array, i as i32, cell_array)?;
            }

            // Create 2D array for proofs
            let proofs_array = env.new_object_array(
                proofs.len() as i32,
                proof_byte_array_class,
                env.new_byte_array(0)?,
            )?;

            for (i, proof) in proofs.iter().enumerate() {
                let proof_array = env.byte_array_from_slice(proof.as_ref())?;
                env.set_object_array_element(&proofs_array, i as i32, proof_array)?;
            }

            // Create the CellsAndProofs object
            let cells_and_proofs_obj = env.new_object(
                cells_and_proofs_class,
                "([[B[[B)V",
                &[JValue::Object(&cells_array), JValue::Object(&proofs_array)],
            )?;

            Ok(cells_and_proofs_obj)
        }

        /// Throws an exception in Java
        fn throw_on_error(env: &mut JNIEnv, err: Error, func_name: &'static str) {
            let reason = match err {
                Error::Jni(err) => format!("{:?}", err),
                Error::IncorrectSize {
                    expected,
                    got,
                    name,
                } => format!("{name} is not the correct size. expected: {expected}\ngot: {got}"),
                Error::Cryptography(err) => format!("{:?}", err),
            };
            let msg = format!(
                "function {} has thrown an exception, with reason: {}",
                func_name, reason
            );
            env.throw_new("java/lang/IllegalArgumentException", msg)
                .expect("Failed to throw exception");
        }

        /// Convert a slice into a reference to an array
        ///
        /// This is needed as the API for rust library does
        /// not accept slices.
        fn slice_to_array_ref<'a, const N: usize>(
            slice: &'a [u8],
            name: &'static str,
        ) -> Result<&'a [u8; N], Error> {
            slice.try_into().map_err(|_| Error::IncorrectSize {
                expected: N,
                got: slice.len(),
                name,
            })
        }
    };
}

// Below types are copied from `blst` crate.
// It is needed so other backends do not depend on blst runtime, but still can
// provide c-kzg-4844 compatible apis.
#[allow(non_camel_case_types)]
pub type limb_t = u64;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct blst_fr {
    pub l: [limb_t; 4usize],
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct blst_p1 {
    pub x: blst_fp,
    pub y: blst_fp,
    pub z: blst_fp,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct blst_fp {
    pub l: [limb_t; 6usize],
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct blst_p2 {
    pub x: blst_fp2,
    pub y: blst_fp2,
    pub z: blst_fp2,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct blst_fp2 {
    pub fp: [blst_fp; 2usize],
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct blst_p1_affine {
    pub x: blst_fp,
    pub y: blst_fp,
}
