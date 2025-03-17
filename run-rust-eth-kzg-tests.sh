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
cd $backend

###################### cloning rust-eth-kzg ######################

print_msg "Removing existing rust-eth-kzg"
rm -rf rust-eth-kzg

print_msg "Cloning rust-eth-kzg"
git clone https://github.com/crate-crypto/rust-eth-kzg.git
cd rust-eth-kzg || exit
git -c advice.detachedHead=false checkout "$RUST_ETH_KZG_GIT_HASH"

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

# ###################### java tests ######################

print_msg "Building java"
./scripts/compile.sh java
cd bindings/java/java_code || exit
./gradlew build

print_msg "Running java tests"
./gradlew test
cd ../../..

# ###################### nodejs tests ######################

cd bindings/node || exit
print_msg "Building nodejs"
yarn install
yarn build

print_msg "Running nodejs tests"
yarn test
cd ../..

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

print_msg "Cleaning up"
rm -rf rust-eth-kzg
