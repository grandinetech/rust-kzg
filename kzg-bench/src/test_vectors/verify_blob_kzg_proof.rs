#![allow(dead_code)]

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Input<'a> {
    blob: &'a str,
    commitment: &'a str,
    proof: &'a str,
}

impl Input<'_> {
    pub fn get_blob_bytes(&self) -> Vec<u8> {
        hex::decode(&self.blob[2..]).unwrap()
    }

    pub fn get_commitment_bytes(&self) -> Vec<u8> {
        hex::decode(&self.commitment[2..]).unwrap()
    }

    pub fn get_proof_bytes(&self) -> Vec<u8> {
        hex::decode(&self.proof[2..]).unwrap()
    }
}

#[derive(Deserialize)]
pub struct Test<'a> {
    #[serde(borrow)]
    pub input: Input<'a>,
    output: Option<bool>,
}

impl Test<'_> {
    pub fn get_output(&self) -> Option<bool> {
        self.output
    }
}
