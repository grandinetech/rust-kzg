#![allow(dead_code)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Input<'a> {
    blob: &'a str,
}

impl Input<'_> {
    pub fn get_blob_bytes(&self) -> Result<Vec<u8>, String> {
        hex::decode(&self.blob[2..]).map_err(|_| "Invalid blob".to_string())
    }
}

#[derive(Deserialize)]
pub struct Test<'a> {
    #[serde(borrow)]
    pub input: Input<'a>,
    output: Option<(Vec<String>, Vec<String>)>,
}

impl Test<'_> {
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
                    .collect::<Vec<Vec<u8>>>(),
            )
        })
    }
}
