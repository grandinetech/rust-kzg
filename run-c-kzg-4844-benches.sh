#!/bin/bash

set -e

print_msg () {
  echo "[*]" "$1"
}

###################### parallel & backend configuration ######################

parallel=false
backend="unknown"

while [[ -n $# ]]; do
  case $1 in
    -p|--parallel)
      parallel=true
      ;;
    blst|arkworks|arkworks3|mcl|zkcrypto|constantine)
      backend="$1"
      ;;
    *)
      break
      ;;
  esac
  shift
done

if [ "$backend" == "unknown" ]; then
  echo "Unknown backend: $backend"
  exit 1
fi

###################### building static libs ######################
print_msg "Selected backend: $backend"

print_msg "Compiling rust-kzg-$backend"
cd $backend

if [[ "$parallel" = true ]]; then
  print_msg "Using parallel version"
  cargo rustc --release --crate-type=staticlib --features=parallel
else
  print_msg "Using non-parallel version"
  cargo rustc --release --crate-type=staticlib
fi

mv ../target/release/librust_kzg_$backend.a ../target/release/rust_kzg_$backend.a

###################### cloning c-kzg-4844 ######################

print_msg "Cloning c-kzg-4844"
git clone https://github.com/ethereum/c-kzg-4844.git
cd c-kzg-4844 || exit
git -c advice.detachedHead=false checkout "$C_KZG_4844_GIT_HASH"
git submodule update --init

###################### rust benchmarks ######################

print_msg "Patching rust binding"
git apply < ../rust.patch
cd bindings/rust || exit

print_msg "Running rust benchmarks"
cargo bench
cd ../..

###################### java benchmarks ######################

print_msg "Patching java binding"
git apply < ../java.patch
cd bindings/java || exit

print_msg "Running java benchmarks"
make build benchmark
cd ../..

###################### go benchmarks ######################

print_msg "Patching go binding"
git apply < ../go.patch
cd bindings/go || exit

print_msg "Running go benchmarks"
CGO_CFLAGS="-O2 -D__BLST_PORTABLE__" go test -run ^$ -bench .
cd ../../..

###################### cleaning up ######################

print_msg "Cleaning up"
rm -rf c-kzg-4844
