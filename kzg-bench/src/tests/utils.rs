use std::{env::current_dir, path::PathBuf};

use kzg::eip_4844::TRUSTED_SETUP_PATH;
use pathdiff::diff_paths;

pub fn get_manifest_dir() -> String {
    let current = current_dir().unwrap();
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let relative = diff_paths(manifest, current).unwrap();

    relative.into_os_string().into_string().unwrap()
}

pub fn get_trusted_setup_path() -> String {
    PathBuf::from(get_manifest_dir())
        .join(TRUSTED_SETUP_PATH)
        .into_os_string()
        .into_string()
        .unwrap()
}
