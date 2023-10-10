

use kzg::common_utils::{load_trusted_setup_rust};

use crate::kzg_proofs::{KZGSettings};




use kzg::eip_4844::{
    load_trusted_setup_string,
};
use kzg::{FFTSettings as FFTSettingsT, KZGSettings as LKZGSettings};
use kzg::{Poly};
use std::fs::File;
use std::io::Read;


#[cfg(feature = "parallel")]
use rayon::prelude::*;

pub fn load_trusted_setup(filepath: &str) -> Result<KZGSettings, String> {
    let mut file = File::open(filepath).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    let (g1_bytes, g2_bytes) = load_trusted_setup_string(&contents)?;
    load_trusted_setup_rust(g1_bytes.as_slice(), g2_bytes.as_slice())
}
