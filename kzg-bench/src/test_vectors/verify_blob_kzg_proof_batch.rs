#![allow(dead_code)]

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Input {
    blobs: Vec<String>,
    commitments: Vec<String>,
    proofs: Vec<String>,
}

impl Input {
    pub fn get_blobs_bytes(&self) -> Vec<Vec<u8>> {
        let mut v = Vec::new();
        for blob in &self.blobs {
            v.push(hex::decode(&blob[2..]).unwrap());
        }
        v
    }

    pub fn get_commitments_bytes(&self) -> Vec<Vec<u8>> {
        let mut v = Vec::new();
        for commitment in &self.commitments {
            v.push(hex::decode(&commitment[2..]).unwrap());
        }
        v
    }

    pub fn get_proofs_bytes(&self) -> Vec<Vec<u8>> {
        let mut v = Vec::new();
        for proof in &self.proofs {
            v.push(hex::decode(&proof[2..]).unwrap());
        }
        v
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
