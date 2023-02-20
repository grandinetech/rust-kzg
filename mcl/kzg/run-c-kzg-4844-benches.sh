#!/bin/bash

set -e

LIB=libmcl_rust.a
SED_LINUX="/usr/bin/sed"
SED_MACOS="/usr/local/bin/gsed"

print_msg () {
  echo "[*]" "$1"
}

###################### parallel configuration ######################

parallel=false

while getopts "parallel" opt; do
  case $opt in
    p)
      parallel=true
      ;;
    \?)
      exit 1
      ;;
  esac
done

###################### building static libs ######################

print_msg "Compiling libmcl_rust"
if [[ "$parallel" = true ]]; then
  print_msg "Using parallel version"
  cargo rustc --release --crate-type=staticlib --features=parallel
else
  print_msg "Using non-parallel version"
  cargo rustc --release --crate-type=staticlib
fi

###################### cloning c-kzg-4844 ######################

print_msg "Cloning c-kzg-4844"
git clone https://github.com/ethereum/c-kzg-4844.git
cd c-kzg-4844 || exit 1
git -c advice.detachedHead=false checkout "$C_KZG_4844_GIT_HASH"
git submodule update --init

print_msg "Applying patches and building blst"
cd src
export CFLAGS="-Ofast -fno-builtin-memcpy -fPIC -Wall -Wextra -Werror"
make blst
unset CFLAGS
cd ..

###################### detecting os ######################

case $(uname -s) in
  "Linux")
    sed=$SED_LINUX
    ;;
  "Darwin")
    if [[ -z $(command -v "$SED_MACOS") ]]; then
      echo "FAIL: gsed was not found"
      echo "HELP: to fix this, run \"brew install gnu-sed\""
      exit 1
    fi
    sed=$SED_MACOS
    ;;
  *)
    echo "FAIL: unsupported OS"
    exit 1
    ;;
esac

###################### rust benchmarks ######################

#print_msg "Patching rust binding"
#git apply < ../rust.patch
#cd bindings/rust || exit 1

#print_msg "Running rust benchmarks"
#cargo bench
#cd ../..

###################### java benchmarks ######################

print_msg "Patching java binding"
cd bindings/java || exit 1
eval "$("$sed" -i "s|../../src/c_kzg_4844.c ../../lib/libblst.a|../../../target/release/$LIB|g" Makefile)"

print_msg "Running java benchmarks"
make CC_FLAGS=-lstdc++ build benchmark
cd ../..

###################### go benchmarks ######################

print_msg "Patching go binding"
git apply < ../go.patch

print_msg "Running go benchmarks"
cd bindings/go || exit 1
go test -run ^$ -bench .
cd ../../..

###################### cleaning up ######################

print_msg "Cleaning up"
rm -rf c-kzg-4844
