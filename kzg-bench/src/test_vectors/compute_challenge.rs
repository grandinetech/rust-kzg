#![allow(dead_code)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Input<'a> {
    blob: &'a str,
    commitment: &'a str,
}

impl Input<'_> {
    pub fn get_blob_bytes(&self) -> Result<Vec<u8>, String> {
        hex::decode(&self.blob[2..]).map_err(|_| "Invalid blob".to_string())
    }

    pub fn get_commitment_bytes(&self) -> Result<Vec<u8>, String> {
        hex::decode(&self.commitment[2..]).map_err(|_| "Invalid commitment".to_string())
    }
}

#[derive(Deserialize)]
pub struct Test<'a> {
    #[serde(borrow)]
    pub input: Input<'a>,
    output: Option<&'a str>,
}

impl Test<'_> {
    pub fn get_output_bytes(&self) -> Option<Vec<u8>> {
        self.output.map(|s| hex::decode(&s[2..]).unwrap())
    }
}
