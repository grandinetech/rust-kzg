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
git -c advice.detachedHead=false checkout d4b40c33ee86c0556eae8d6aa0380f926911e9ba

print_msg "Building blst"
bash build.sh
cd ..

print_msg "Cloning c-kzg"
git clone https://github.com/tesa4436/c-kzg

print_msg "Copying files from blst to c-kzg"
cp -r blst/* c-kzg/lib/
cp -r blst/bindings/*.h c-kzg/inc/

print_msg "Preparing c-kzg's makefile"
cd c-kzg/src/ || exit 1
sed=""
case $(uname -s) in
  "Linux")
    sed=$(sed_linux)
    ;;
  "Darwin")
    sed=$(sed_macos)
    ;;
  *)
    echo "ERR: Unsupported OS"
    exit 1
    ;;
esac
eval "$("$sed" -i 's/KZG_CFLAGS =/KZG_CFLAGS = -fPIE/' Makefile)"

print_msg "Building c-kzg"
make lib

print_msg "Preparing ckzg crate"
cd ../../
cp c-kzg/lib/libblst.a lib/
cp c-kzg/src/libckzg.a lib/

print_msg "Cleaning up"
rm -rf blst/
rm -rf c-kzg/
