use std::env;
use std::process::Command;

fn main() {
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();

    if !Command::new("sh")
        .arg(format!("{}/build.sh", cargo_manifest_dir))
        .current_dir(out_dir.clone())
        .status()
        .expect("Failed to build")
        .success()
    {
        panic!("Built script failed");
    }

    println!("cargo:rustc-link-search={}/lib", out_dir);
}
