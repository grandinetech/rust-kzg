#![allow(dead_code)]

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Input<'a> {
    blob: &'a str,
    commitment: &'a str,
}

impl Input<'_> {
    pub fn get_blob_bytes(&self) -> Vec<u8> {
        hex::decode(&self.blob[2..]).unwrap()
    }

    pub fn get_commitment_bytes(&self) -> Vec<u8> {
        hex::decode(&self.commitment[2..]).unwrap()
    }
}

#[derive(Deserialize)]
pub struct Test<'a> {
    #[serde(borrow)]
    pub input: Input<'a>,
    #[serde(borrow)]
    output: Option<&'a str>,
}

impl Test<'_> {
    pub fn get_output_bytes(&self) -> Option<Vec<u8>> {
        self.output.map(|s| hex::decode(&s[2..]).unwrap())
    }
}
