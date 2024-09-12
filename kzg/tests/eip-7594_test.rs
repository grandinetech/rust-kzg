use crate::compute_cells_and_kzg_proofs;
use crate::types::{Blob, Cell, KZGProof, S};
use kzg::eip_4844::{compute_cells_and_kzg_proofs_rust, CELLS_PER_EXT_BLOB};
use rand::Rng; // Adjust imports as necessary

#[test]

fn test_compute_cells_and_kzg_proofs() {
    let mut blob = Blob::default();
    let mut cells = [Cell::default(); CELLS_PER_EXT_BLOB];
    let mut proofs: [KZGProof::default(); CELLS_PER_EXT_BLOB];
    let s = S::default(); // Assuming 's' is an instance of some struct

    // Randomly generate the blob
    let mut rng = rand::thread_rng();
    for i in 0..blob.data.len() {
        blob.data[i] = rng.gen(); // Assuming Blob has a data field that's a byte array
    }
    for _ in 0..5 {
        let ret = compute_cells_and_kzg_proofs_rust(blob, s);
        assert_eq!(ret, C_KZG_RET_OK); // Replace `C_KZG_OK` with the appropriate success constant or value in Rust
    }
}
