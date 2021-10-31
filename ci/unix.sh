#!/bin/bash

print_msg () {
  echo "[*]" "$1"
}

print_msg "Removing old libs"
rm -rf kzg-bindings/lib/*

print_msg "Cloning blst"
git clone https://github.com/supranational/blst.git
cd blst || exit
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
cd c-kzg/src/ || exit
eval "$("$1" -i 's/KZG_CFLAGS =/KZG_CFLAGS = -fPIE/' Makefile)"

print_msg "Building c-kzg"
make lib

print_msg "Preparing kzg-bindings's crate"
cd ../../
cp c-kzg/lib/libblst.a kzg-bindings/lib/
cp c-kzg/src/libckzg.a kzg-bindings/lib/
