fn main() {
  println!("cargo:rustc-link-search=./lib");
  println!("cargo:rustc-link-lib=ckzg");
  println!("cargo:rustc-link-lib=blst");
}
