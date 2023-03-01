use std::{env, fs};
use std::process::Command;

fn main() {
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();

    fs::copy(
        format!("{cargo_manifest_dir}/0001-Bring-back-the-bytes-conversion-functions.patch"),
        format!("{out_dir}/0001-Bring-back-the-bytes-conversion-functions.patch"),
    ).unwrap();

    if !Command::new("sh")
        .arg(format!("{}/build.sh", cargo_manifest_dir))
        .current_dir(out_dir.clone())
        .status()
        .expect("Failed to build")
        .success() {
        panic!("Built script failed");
    }

    println!("cargo:rustc-link-search={}/lib", out_dir);
    println!("cargo:rustc-link-lib=static=ckzg");
    println!("cargo:rustc-link-lib=static=blst");
}
