#!/bin/bash
unset CARGO_TARGET_DIR

set -e

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

SED_LINUX="/usr/bin/sed"
SED_MACOS="/usr/local/bin/gsed"

print_msg () {
  echo "[*]" "$1"
}

LIB=libmcl_rust.a

print_msg "Compiling lib"
if [[ "$parallel" = true ]]; then
  print_msg "Using parallel version"
  cargo rustc --release --crate-type=staticlib --features=parallel
else
  print_msg "Using non-parallel version"
  cargo rustc --release --crate-type=staticlib
fi

print_msg "Cloning c-kzg-4844"
git clone https://github.com/ethereum/c-kzg-4844.git
cd c-kzg-4844 || exit 1
git checkout $C_KZG_4844_GIT_HASH
git submodule update --init

print_msg "Coping header files"
cd blst
cp bindings/*.h ../inc
cd ..

case $(uname -s) in
  "Linux")
    sed=$SED_LINUX
    CSHARP_PLATFORM=linux-x64
    CLANG_PLATFORM=x86_64-linux
    ;;
  "Darwin")
    if [[ -z $(command -v "$SED_MACOS") ]]; then
      echo "FAIL: gsed was not found"
      echo "HELP: to fix this, run \"brew install gnu-sed\""
      exit 1
    fi
    sed=$SED_MACOS
    CSHARP_PLATFORM=osx-x64
    CLANG_PLATFORM=x86_64-darwin
    ;;
  *)
    echo "FAIL: unsupported OS"
    exit 1
    ;;
esac

# print_msg "Modyfying dotnet Makefile"
# git apply < ../csharp.patch

# print_msg "Building dotnet"
# cd bindings/csharp
# make -B ckzg CSHARP_PLATFORM=$CSHARP_PLATFORM CLANG_PLATFORM=$CLANG_PLATFORM
# dotnet restore

# print_msg "Running dotnet tests"
# dotnet test --configuration Release --no-restore
# cd ../..

# print_msg "Modyfing rust bindings build.rs"
# git apply < ../rust.patch
# cd bindings/rust || exit 1

# print_msg "Running rust tests"
# cargo test --release
# cd ../..

print_msg "Modyfing python bindings makefile"
cd bindings/python || exit 1

eval "$("$sed" -i "s|../../src/c_kzg_4844.o ../../lib/libblst.a|../../../target/release/$LIB|g" Makefile)"
eval "$("$sed" -i "s|-Wall -shared|-Wall -lstdc++ -shared|g" Makefile)"

print_msg "Running python tests"
make
cd ../..

print_msg "Modyfing java bindings makefile"
cd bindings/java || exit 1
eval "$("$sed" -i "s|../../src/c_kzg_4844.c ../../lib/libblst.a|../../../target/release/$LIB|g" Makefile)"

print_msg "Running java tests and benchmarks"
make CC_FLAGS=-lstdc++ build test
cd ../..

print_msg "Modyfing nodejs bindings"
cd bindings/node.js || exit 1
eval "$("$sed" -i "s/c_kzg_4844.o/..\/..\/..\/target\/release\/$LIB/g" binding.gyp)"
eval "$("$sed" -i 's|"<(module_root_dir)/../../lib/libblst.a"||g' binding.gyp)"
eval "$("$sed" -i '/cd ..\/..\/src; make lib/c\\t# cd ..\/..\/src; make lib' Makefile)"

print_msg "Running nodejs tests"
yarn install
make
cd ../../..

print_msg "Cleaning up"
rm -rf c-kzg-4844
