#!/bin/bash

set -e

LIB=libblst_from_scratch.a
SED_LINUX="/usr/bin/sed"
SED_MACOS="/usr/local/bin/gsed"

print_msg () {
  echo "[*]" "$1"
}

###################### parallel configuration ######################

parallel=false

while [[ -n $# ]]; do
  case $1 in
    -p|--parallel)
      parallel=true
      ;;
    -*)
      echo "Unknown parameter: $1"
      exit 1
      ;;
    *)
      break
      ;;
  esac;
  shift
done

###################### building static libs ######################

print_msg "Compiling libblst_from_scratch"
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

###################### c tests ######################

print_msg "Patching c tests"
git apply < ../c.patch

print_msg "Running c tests"
cd src
make test
cd ..

###################### dotnet tests ######################

print_msg "Patching dotnet binding"
git apply < ../csharp.patch

print_msg "Building dotnet"
cd bindings/csharp
make -B ckzg CSHARP_PLATFORM=$CSHARP_PLATFORM CLANG_PLATFORM=$CLANG_PLATFORM
dotnet restore

print_msg "Running dotnet tests"
dotnet test --configuration Release --no-restore
cd ../..

###################### rust tests ######################

print_msg "Patching rust binding"
git apply < ../rust.patch
cd bindings/rust || exit 1

print_msg "Running rust tests"
cargo test --release
cd ../..

print_msg "Rebuilding blst"
cd src
export CFLAGS="-Ofast -fno-builtin-memcpy -fPIC -Wall -Wextra -Werror"
make blst
unset CFLAGS
cd ..

###################### python tests ######################

print_msg "Patching python binding"
git apply < ../python.patch
cd bindings/python || exit 1

print_msg "Running python tests"
make
cd ../..

###################### java tests ######################

print_msg "Patching java binding"
git apply < ../java.patch
cd bindings/java || exit 1

print_msg "Running java tests"
make build test
cd ../..

###################### nodejs tests ######################

print_msg "Patching nodejs binding"
git apply < ../nodejs.patch
cd bindings/node.js || exit 1

print_msg "Running nodejs tests"
yarn install
make
cd ../..

###################### go tests ######################

print_msg "Patching go binding"
git apply < ../go.patch

print_msg "Running go tests"
cd bindings/go || exit 1
go test .
cd ../../..

###################### cleaning up ######################

print_msg "Cleaning up"
rm -rf c-kzg-4844
