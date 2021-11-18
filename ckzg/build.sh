#!/bin/bash

sed_linux="/usr/bin/sed"
sed_macos="/usr/local/bin/gsed"

print_msg () {
  echo "[*]" "$1"
}

print_msg "Removing old libs"
rm -rf lib/*
mkdir -p lib

print_msg "Cloning blst"
git clone https://github.com/supranational/blst.git
cd blst || exit 1

print_msg "Building blst"
export CFLAGS="-Ofast -fno-builtin-memcpy -fPIC -Wall -Wextra -Werror"
bash build.sh
unset CFLAGS
cd ..

print_msg "Cloning c-kzg"
git clone --single-branch --branch openmp https://github.com/tesa4436/c-kzg

print_msg "Copying files from blst to c-kzg"
cp -r blst/* c-kzg/lib/
cp -r blst/bindings/*.h c-kzg/inc/

print_msg "Preparing c-kzg's makefile"
cd c-kzg/src/ || exit 1
sed=""
case $(uname -s) in
  "Linux")
    sed=$sed_linux
    ;;
  "Darwin")
    if [[ -z $(command -v "$sed_macos") ]]; then
      echo "FAIL: gsed was not found"
      echo "HELP: to fix this, run \"brew install gnu-sed\""
      exit 1
    fi
    sed=$sed_macos
    ;;
  *)
    echo "FAIL: unsupported OS"
    exit 1
    ;;
esac
eval "$("$sed" -i 's/KZG_CFLAGS =/KZG_CFLAGS = -fPIE -fopenmp/' Makefile)"
eval "$("$sed" -i 's/KZG_CFLAGS += -O/KZG_CFLAGS += -Ofast/' Makefile)"

print_msg "Building c-kzg"
make lib

print_msg "Preparing ckzg crate"
cd ../../
cp c-kzg/lib/libblst.a lib/
cp c-kzg/src/libckzg.a lib/

print_msg "Cleaning up"
rm -rf blst/
rm -rf c-kzg/
