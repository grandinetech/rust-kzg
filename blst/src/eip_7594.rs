extern crate alloc;

use kzg::EcBackend;

use crate::types::fft_settings::FsFFTSettings;
use crate::types::fp::FsFp;
use crate::types::g1::FsG1;
use crate::types::g1::FsG1Affine;
use crate::types::g1::FsG1ProjAddAffine;
use crate::types::g2::FsG2;
use crate::types::kzg_settings::FsKZGSettings;
use crate::types::poly::FsPoly;

use crate::types::fr::FsFr;

pub struct BlstBackend;

impl EcBackend for BlstBackend {
    type Fr = FsFr;
    type G1Fp = FsFp;
    type G1Affine = FsG1Affine;
    type G1 = FsG1;
    type G2 = FsG2;
    type Poly = FsPoly;
    type FFTSettings = FsFFTSettings;
    type KZGSettings = FsKZGSettings;
    type G1ProjAddAffine = FsG1ProjAddAffine;
}

#[cfg(feature = "c_bindings")]
kzg::c_bindings_eip7594!(BlstBackend);

#[cfg(feature = "c_bindings")]
#[no_mangle]
pub unsafe extern "C" fn compute_verify_cell_kzg_proof_batch_challenge(
    challenge_out: *mut blst::blst_fr,
    commitment_bytes: *const kzg::eth::c_bindings::Bytes48,
    num_commitments: u64,
    commitment_indices: *const u64,
    cell_indices: *const u64,
    cells: *const kzg::eth::c_bindings::Cell,
    proofs_bytes: *const kzg::eth::c_bindings::Bytes48,
    num_cells: u64,
) -> kzg::eth::c_bindings::CKzgRet {
    use crate::handle_ckzg_badargs;
    use kzg::{eip_4844::BYTES_PER_FIELD_ELEMENT, Fr, G1};

    *challenge_out = blst::blst_fr::default();

    let commitment_bytes =
        unsafe { core::slice::from_raw_parts(commitment_bytes, num_commitments as usize) };
    let commitments = handle_ckzg_badargs!(commitment_bytes
        .iter()
        .map(|v| FsG1::from_bytes(&v.bytes))
        .collect::<Result<Vec<_>, _>>());

    let commitment_indices =
        unsafe { core::slice::from_raw_parts(commitment_indices, num_cells as usize) };
    let commitment_indices = commitment_indices
        .iter()
        .map(|v| *v as usize)
        .collect::<Vec<_>>();

    let cell_indices = unsafe { core::slice::from_raw_parts(cell_indices, num_cells as usize) };
    let cell_indices = cell_indices.iter().map(|c| *c as usize).collect::<Vec<_>>();

    let cells = unsafe { core::slice::from_raw_parts(cells, num_cells as usize) };
    let cells = handle_ckzg_badargs!(cells
        .iter()
        .flat_map(|c| c
            .bytes
            .chunks(BYTES_PER_FIELD_ELEMENT)
            .map(|bytes| FsFr::from_bytes(&bytes)))
        .collect::<Result<Vec<_>, _>>());

    let proofs_bytes = unsafe { core::slice::from_raw_parts(proofs_bytes, num_cells as usize) };
    let proofs = handle_ckzg_badargs!(proofs_bytes
        .iter()
        .map(|b| FsG1::from_bytes(&b.bytes))
        .collect::<Result<Vec<_>, _>>());

    let challenge = handle_ckzg_badargs!(
        <FsKZGSettings as kzg::DAS<BlstBackend>>::compute_verify_cell_kzg_proof_batch_challenge(
            kzg::eth::FIELD_ELEMENTS_PER_CELL,
            &commitments,
            &commitment_indices,
            &cell_indices,
            &cells,
            &proofs,
            kzg::eth::FIELD_ELEMENTS_PER_BLOB
        )
    );

    *challenge_out = challenge.0;

    kzg::eth::c_bindings::CKzgRet::Ok
}
