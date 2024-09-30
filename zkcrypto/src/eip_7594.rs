extern crate alloc;

use alloc::vec::Vec;
use crate::kzg_types::{Fr, G1, Bytes48, C_KZG_OK, C_KZG_RET};
use crate::kzg_proofs::g1_lincomb_fast;
use crate::common_utils::bytes_to_kzg_commitment;

pub fn compute_weighted_sum_of_commitments(
    sum_of_commitments_out: &mut G1,
    unique_commitments: &[Bytes48],
    commitment_indices: &[u64],
    r_powers: &[Fr],
    num_commitments: usize,
    num_cells: u64,
) -> C_KZG_RET {
    let mut commitments_g1: Vec<G1> = Vec::with_capacity(num_commitments);
    let mut commitment_weights: Vec<Fr> = vec![Fr::zero(); num_commitments];

    // Convert unique commitments to G1 points and validate them
    for i in 0..num_commitments {
        let commitment = bytes_to_kzg_commitment(&unique_commitments[i]);
        match commitment {
            Ok(g1) => commitments_g1.push(g1),
            Err(ret) => return ret,
        }
    }

    // Update commitment weights based on commitment_indices and r_powers
    for i in 0..num_cells as usize {
        let idx = commitment_indices[i] as usize;
        commitment_weights[idx] += r_powers[i];
    }

    // Compute the linear combination of commitments with the calculated weights
    match g1_lincomb_fast(sum_of_commitments_out, &commitments_g1, &commitment_weights, num_commitments) {
        Ok(_) => C_KZG_OK,
        Err(ret) => ret,
    }
}
