#![allow(dead_code)]

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Input<'a> {
    blob: &'a str,
    z: &'a str,
}

impl Input<'_> {
    pub fn get_blob_bytes(&self) -> Vec<u8> {
        hex::decode(&self.blob[2..]).unwrap()
    }

    pub fn get_z_bytes(&self) -> Vec<u8> {
        hex::decode(&self.z[2..]).unwrap()
    }
}

#[derive(Deserialize)]
pub struct Test<'a> {
    #[serde(borrow)]
    pub input: Input<'a>,
    #[serde(borrow)]
    output: Option<(&'a str, &'a str)>,
}

impl Test<'_> {
    pub fn get_output_bytes(&self) -> Option<(Vec<u8>, Vec<u8>)> {
        self.output.map(|(proof, y)| {
            (
                hex::decode(&proof[2..]).unwrap(),
                hex::decode(&y[2..]).unwrap(),
            )
        })
    }
}
