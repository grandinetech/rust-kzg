#!/bin/bash

# current checkout hash
C_KZG_4844_GIT_HASH=0894e3ded53d6f85f5aa146f4fe3e80afb411b94

set -e

print_msg () {
  echo "[*]" "$1"
}

###################### parallel & backend configuration ######################

parallel=false
use_arkmsm=false
use_bgmw=false
backend="unknown"

while [[ -n $# ]]; do
  case $1 in
    -p|--parallel)
      parallel=true
      ;;
    --arkmsm)
      use_arkmsm=true
      ;;
    --bgmw)
      use_bgmw=true
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

features="c_bindings"

#Check if --arkmsm is specified
if [ "$use_arkmsm" = true ]; then
  features+=",arkmsm"
fi

# Check if --bgmw is specified
if [ "$use_bgmw" = true ]; then
  features+=",bgmw"
fi

# Trim the trailing comma, if any
features=$(echo "$features" | sed 's/,$//')

###################### building static libs ######################
print_msg "Compiling rust-kzg-$backend"
cd $backend

if [[ "$parallel" = true ]]; then
  print_msg "Using parallel version"
  cargo rustc --release --crate-type=staticlib --features="$features,parallel"
else
  print_msg "Using non-parallel version"
  cargo rustc --release --crate-type=staticlib --features="$features"
fi

mv ../target/release/librust_kzg_$backend.a ../target/release/rust_kzg_$backend.a

###################### cloning c-kzg-4844 ######################

print_msg "Removing existing c-kzg-4844"
rm -rf c-kzg-4844

print_msg "Cloning c-kzg-4844"
git clone https://github.com/ethereum/c-kzg-4844.git
cd c-kzg-4844 || exit
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

###################### fuzzing ######################

pwd
print_msg "Patching go binding"
git apply < ../go.patch
cd bindings/go || exit

git clone https://github.com/jtraglia/kzg-fuzz.git
cd kzg-fuzz || exit

go clean -modcache

echo 'replace "github.com/ethereum/c-kzg-4844" v0.3.1 => "../../../"' >> go.mod

go install

bash fuzz.sh

# ###################### cleaning up ######################

print_msg "Cleaning up"
rm -rf c-kzg-4844