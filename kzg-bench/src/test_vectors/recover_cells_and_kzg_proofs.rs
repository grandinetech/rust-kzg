#![allow(dead_code)]

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Input {
    cell_indices: Vec<u8>,
    cells: Vec<String>,
}

impl Input {
    pub fn get_cell_indices(&self) -> Result<Vec<u8>, String> {
        Ok(self.cell_indices.clone())
    }

    pub fn get_cell_bytes(&self) -> Result<Vec<Vec<u8>>, String> {
        self.cells
            .iter()
            .map(|s| hex::decode(&s[2..]).map_err(|_| "Invalid cell hex".to_string()))
            .collect::<Result<Vec<Vec<u8>>, String>>()
    }
}

#[derive(Deserialize)]
pub struct Test {
    pub input: Input,
    output: Option<(Vec<String>, Vec<String>)>,
}

impl Test {
    #[allow(clippy::type_complexity)]
    pub fn get_output(&self) -> Option<(Vec<Vec<u8>>, Vec<Vec<u8>>)> {
        self.output.clone().map(|(cells, proofs)| {
            (
                cells
                    .iter()
                    .map(|s| hex::decode(&s[2..]).unwrap())
                    .collect::<Vec<_>>(),
                proofs
                    .iter()
                    .map(|s| hex::decode(&s[2..]).unwrap())
                    .collect::<Vec<_>>(),
            )
        })
    }
}
