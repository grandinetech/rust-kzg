#!/bin/bash

set -e

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

print_msg "Cloning 4844"
git clone https://github.com/ethereum/c-kzg-4844.git
cd c-kzg-4844 || exit 1
git -c advice.detachedHead=false checkout $C_KZG_4844_GIT_HASH
git apply < ../c_kzg_4844.patch

print_msg "Cloning blst"
git submodule update --init

print_msg "Applying patches and building blst"
cd src
export CFLAGS="-Ofast -fno-builtin-memcpy -fPIC -Wall -Wextra -Werror"
make blst
unset CFLAGS

cd ../..
print_msg "Cloning c-kzg"
git clone --single-branch --branch openmp https://github.com/tesa4436/c-kzg

print_msg "Copying files from blst to c-kzg"
cp -r c-kzg-4844/blst/* c-kzg/lib/
cp -r c-kzg-4844/blst/bindings/*.h c-kzg/inc/

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

print_msg "Building 4844"
cd ../../c-kzg-4844/src || exit 1

make c_kzg_4844.o
ar rc ../../c-kzg/src/libckzg.a c_kzg_4844.o

print_msg "Preparing ckzg crate"
cd ../../
cp c-kzg/lib/libblst.a lib/
cp c-kzg/src/libckzg.a lib/

print_msg "Cleaning up"
rm -rf c-kzg-4844/
rm -rf c-kzg/
