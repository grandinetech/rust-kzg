#![allow(dead_code)]

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Input {
    commitments: Vec<String>,
    cell_indices: Vec<u64>,
    cells: Vec<String>,
    proofs: Vec<String>,
}

impl Input {
    pub fn get_commitment_bytes(&self) -> Result<Vec<Vec<u8>>, String> {
        self.commitments
            .iter()
            .map(|s| hex::decode(&s[2..]).map_err(|_| "Failed to decode hex".to_string()))
            .collect::<Result<Vec<Vec<u8>>, String>>()
    }

    pub fn get_cell_indices(&self) -> Result<Vec<usize>, String> {
        Ok(self
            .cell_indices
            .iter()
            .map(|it| *it as usize)
            .collect::<Vec<_>>())
    }

    pub fn get_cell_bytes(&self) -> Result<Vec<Vec<u8>>, String> {
        self.cells
            .iter()
            .map(|s| hex::decode(&s[2..]).map_err(|_| "Failed to decode hex".to_string()))
            .collect::<Result<Vec<Vec<u8>>, String>>()
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
    output: Option<bool>,
}

impl Test {
    pub fn get_output(&self) -> Option<bool> {
        self.output
    }
}
