#![allow(dead_code)]

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Input<'a> {
    commitment: &'a str,
    z: &'a str,
    y: &'a str,
    proof: &'a str,
}

impl Input<'_> {
    pub fn get_commitment_bytes(&self) -> Vec<u8> {
        hex::decode(&self.commitment[2..]).unwrap()
    }

    pub fn get_z_bytes(&self) -> Vec<u8> {
        hex::decode(&self.z[2..]).unwrap()
    }

    pub fn get_y_bytes(&self) -> Vec<u8> {
        hex::decode(&self.y[2..]).unwrap()
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
