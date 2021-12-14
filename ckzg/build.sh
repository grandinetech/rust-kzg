#!/bin/bash

SED_LINUX="/usr/bin/sed"
SED_MACOS="/usr/local/bin/gsed"
OPENMP_LINUX="-fopenmp"
OPENMP_MACOS="-Xpreprocessor -fopenmp"

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
openmp=""
case $(uname -s) in
  "Linux")
    sed=$SED_LINUX
    openmp=$OPENMP_LINUX
    ;;
  "Darwin")
    if [[ -z $(command -v "$SED_MACOS") ]]; then
      echo "FAIL: gsed was not found"
      echo "HELP: to fix this, run \"brew install gnu-sed\""
      exit 1
    fi
    sed=$SED_MACOS
    openmp=$OPENMP_MACOS
    ;;
  *)
    echo "FAIL: unsupported OS"
    exit 1
    ;;
esac
eval "$("$sed" -i "s/KZG_CFLAGS =/KZG_CFLAGS = -fPIE $openmp/" Makefile)"
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
