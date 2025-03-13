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
    blst|arkworks5|arkworks4|arkworks3|mcl|zkcrypto|constantine)
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

# ###################### building static libs ######################
# print_msg "Compiling rust-kzg-$backend"
cd $backend

# if [[ "$parallel" = true ]]; then
#   print_msg "Using parallel version"
#   cargo rustc --release --crate-type=staticlib --features=c_bindings,parallel
# else
#   print_msg "Using non-parallel version"
#   cargo rustc --release --crate-type=staticlib --features=c_bindings
# fi

###################### cloning rust-eth-kzg ######################

print_msg "Removing existing rust-eth-kzg"
rm -rf rust-eth-kzg

print_msg "Cloning rust-eth-kzg"
git clone https://github.com/crate-crypto/rust-eth-kzg.git
cd rust-eth-kzg || exit
git -c advice.detachedHead=false checkout "$RUST_ETH_KZG_GIT_HASH"
# git submodule update --init

echo "Dir: $(pwd)"

print_msg "Patching bindings"
git apply < ../patches/rust-eth-kzg/bindings.patch

print_msg "Compiling libraries"
./scripts/compile.sh

###################### detecting os ######################

case $(uname -s) in
  "Linux")
    CSHARP_PLATFORM=linux-x64
    CLANG_PLATFORM=x86_64-linux
    ;;
  "Darwin")
    CSHARP_PLATFORM=osx-x64
    CLANG_PLATFORM=x86_64-darwin
    ;;
  *)
    echo "FAIL: unsupported OS"
    exit 1
    ;;
esac

###################### dotnet tests ######################

cd bindings/csharp/csharp_code || exit

print_msg "Building dotnet"
# make -B ckzg CSHARP_PLATFORM=$CSHARP_PLATFORM CLANG_PLATFORM=$CLANG_PLATFORM
dotnet restore

print_msg "Running dotnet tests"
dotnet test -c release --no-restore
cd ../../..

# ###################### rust tests ######################

# print_msg "Patching rust binding"
# git apply < ../rust.patch

# print_msg "Running rust tests"
# cargo test --release

# print_msg "Rebuilding blst"
# cd src
# export CFLAGS="-Ofast -fno-builtin-memcpy -fPIC -Wall -Wextra -Werror"
# make blst
# unset CFLAGS
# cd ..

# ###################### python tests ######################

# print_msg "Patching python binding"
# git apply < ../python.patch
# cd bindings/python || exit

# print_msg "Running python tests"
# make
# cd ../..

# ###################### java tests ######################

# print_msg "Patching java binding"
# git apply < ../java.patch
# cd bindings/java || exit

# print_msg "Running java tests"
# make build test
# cd ../..

# ###################### nodejs tests ######################

# print_msg "Patching nodejs binding"
# git apply < ../nodejs.patch
# cd bindings/node.js || exit

# print_msg "Running nodejs tests"
# make
# cd ../..

# ###################### go tests ######################

# print_msg "Patching go binding"
# git apply < ../go.patch
# cd bindings/go || exit

# print_msg "Running go tests"
# CGO_CFLAGS="-O2 -D__BLST_PORTABLE__" go test
# cd ../..

###################### nim tests ######################

if [ "$backend" != "constantine" ]; then
  print_msg "Patching nim binding"
  # git apply < ../nim.patch

  print_msg "Installing nim dependencies"
  # nimble install -y stew
  # nimble install -y unittest2
  # nimble install -y yaml

  print_msg "Running nim tests"
  cd bindings/nim/nim_code || exit
  nimble test
  cd ../../../..
else
  print_msg "Currently, nim bindings are not supported for constantine backend"
fi

# ###################### cleaning up ######################

# print_msg "Cleaning up"
# rm -rf c-kzg-4844
