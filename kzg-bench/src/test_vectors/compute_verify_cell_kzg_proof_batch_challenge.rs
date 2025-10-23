#![allow(dead_code)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Input {
    commitments: Vec<String>,
    commitment_indices: Vec<u64>,
    cell_indices: Vec<u64>,
    cosets_evals: Vec<Vec<String>>,
    proofs: Vec<String>,
}

impl Input {
    pub fn get_commitment_bytes(&self) -> Result<Vec<Vec<u8>>, String> {
        self.commitments
            .iter()
            .map(|s| hex::decode(&s[2..]).map_err(|_| "Failed to decode hex".to_string()))
            .collect::<Result<Vec<Vec<u8>>, String>>()
    }

    pub fn get_commitment_indices(&self) -> Result<Vec<usize>, String> {
        Ok(self
            .commitment_indices
            .iter()
            .map(|it| *it as usize)
            .collect::<Vec<_>>())
    }

    pub fn get_cell_indices(&self) -> Result<Vec<usize>, String> {
        Ok(self
            .cell_indices
            .iter()
            .map(|it| *it as usize)
            .collect::<Vec<_>>())
    }

    pub fn get_coset_eval_bytes(&self) -> Result<Vec<Vec<Vec<u8>>>, String> {
        self.cosets_evals
            .iter()
            .map(|s| {
                s.iter()
                    .map(|s| hex::decode(&s[2..]).map_err(|_| "Failed to decode hex".to_string()))
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, String>>()
    }

    pub fn get_proof_bytes(&self) -> Result<Vec<Vec<u8>>, String> {
        self.proofs
            .iter()
            .map(|s| hex::decode(&s[2..]).map_err(|_| "Failed to decode hex".to_string()))
            .collect::<Result<Vec<Vec<u8>>, String>>()
    }
}

#[derive(Deserialize)]
pub struct Test {
    pub input: Input,
    output: Option<String>,
}

impl Test {
    pub fn get_output_bytes(&self) -> Option<Vec<u8>> {
        self.output.clone().map(|s| hex::decode(&s[2..]).unwrap())
    }
}
