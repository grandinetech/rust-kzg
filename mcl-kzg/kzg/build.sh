#!/bin/bash

BUILD_DIR=${1:-build}

print_msg "Removing old mcl lib"
rm -rf lib/*
mkdir -p lib

print_msg "Cloning mcl"
git clone https://github.com/herumi/mcl.git
cd mcl || exit 1

print_msg "Building mcl"
make -f Makefile MCL_USE_GMP=0

print_msg "Preparing ckzg crate"
cd ..
cp mcl/lib/libmcl.a lib/
cp mcl/lib/libmclbn384_256.a lib/

print_msg "Cleaning up"
rm -rf mcl/