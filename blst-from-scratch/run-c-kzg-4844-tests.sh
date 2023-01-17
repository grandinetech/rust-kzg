#!/bin/bash

SED_LINUX="/usr/bin/sed"
SED_MACOS="/usr/local/bin/gsed"

print_msg () {
  echo "[*]" "$1"
}

print_msg "Compiling libblst_from_scratch"
cargo rustc --release --crate-type=staticlib --features=parallel

print_msg "Cloning c-kzg-4844"
git clone https://github.com/ethereum/c-kzg-4844.git
cd c-kzg-4844 || exit 1

print_msg "Cloning blst"
git submodule update --init

print_msg "Applying patches and building blst"
cd src
export CFLAGS="-Ofast -fno-builtin-memcpy -fPIC -Wall -Wextra -Werror"
make blst
unset CFLAGS
cd ..

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
print_msg "Modyfing java bindings makefile"
cd bindings/java || exit 1
eval "$("$sed" -i "s/..\/..\/src\/c_kzg_4844.c/..\/..\/..\/target\/release\/libblst_from_scratch.a/g" Makefile)"

print_msg "Running java tests and benchmarks"
make
cd ../..

print_msg "Modyfing nodejs bindings"

# There is a bug in node.js when generating trusted setup a newline is missing when changing from G1 points to G2.
git apply < ../node.patch
cd bindings/node.js || exit 1

eval "$("$sed" -i "s/c_kzg_4844.o/..\/..\/..\/target\/release\/libblst_from_scratch.a/g" binding.gyp)"
eval "$("$sed" -i '/cd ..\/..\/src; make lib/c\\t# cd ..\/..\/src; make lib' Makefile)"

print_msg "Running nodejs tests"
yarn install
make
cd ../../..

print_msg "Cleaning up"
rm -rf c-kzg-4844
