use std::{env::set_current_dir, path::Path};

use kzg::eip_4844::TRUSTED_SETUP_PATH;

pub mod benches;
pub mod test_vectors;
pub mod tests;

pub fn set_trusted_setup_dir() {
    set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
}