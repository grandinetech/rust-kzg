use std::env;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let top_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let opt = if cfg!(target_arch = "x86_64") {
        ""
    } else {
        "-DCMAKE_CXX_COMPILER=clang++"
    };

    let cmd = format!(
        "cd {out} && cmake {top}/mcl -DMCL_STATIC_LIB=ON -DMCL_STANDALONE=ON {opt} && make -j",
        out = out_dir,
        top = top_dir,
        opt = opt
    );
    Command::new("sh")
        .args(["-c", &cmd])
        .output()
        .expect("fail");
    let s = format!("cargo:rustc-link-search=native={}/lib", out_dir);
    println!("{}", s);
}
