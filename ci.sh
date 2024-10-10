#!/bin/bash

set -ex

cargo clippy --manifest-path kzg/Cargo.toml --all-targets --no-default-features --features=parallel,std,rand -- -D warnings
cargo clippy --manifest-path kzg/Cargo.toml --all-targets --features=parallel,std,rand,bgmw -- -D warnings
cargo clippy --manifest-path kzg/Cargo.toml --all-targets --features=parallel,std,rand,arkmsm -- -D warnings
cargo clippy --manifest-path kzg/Cargo.toml --all-targets --features=parallel,std,rand,sppark -- -D warnings
cargo fmt --manifest-path kzg/Cargo.toml -- --check
cargo clippy --manifest-path kzg-bench/Cargo.toml --all-targets --all-features -- -D warnings
cargo fmt --manifest-path kzg-bench/Cargo.toml -- --check

cargo clippy --manifest-path blst/Cargo.toml --all-targets --features=default,std,rand,parallel -- -D warnings
cargo fmt --manifest-path blst/Cargo.toml -- --check
cargo clippy --manifest-path blst/Cargo.toml --target wasm32-unknown-unknown --no-default-features
cargo build --manifest-path blst/Cargo.toml --target wasm32-unknown-unknown --no-default-features

cargo clippy --manifest-path arkworks/Cargo.toml --all-targets --features=default,std,rand,parallel -- -D warnings
cargo fmt --manifest-path arkworks/Cargo.toml -- --check
cargo clippy --manifest-path arkworks/Cargo.toml --target wasm32-unknown-unknown --no-default-features
cargo build --manifest-path arkworks/Cargo.toml --target wasm32-unknown-unknown --no-default-features

cargo clippy --manifest-path arkworks3/Cargo.toml --all-targets --features=default,std,rand,parallel -- -D warnings
cargo fmt --manifest-path arkworks3/Cargo.toml -- --check
cargo clippy --manifest-path arkworks3/Cargo.toml --target wasm32-unknown-unknown --no-default-features
cargo build --manifest-path arkworks3/Cargo.toml --target wasm32-unknown-unknown --no-default-features

cargo clippy --manifest-path constantine/Cargo.toml --all-targets --features=default,std,rand,parallel -- -D warnings
cargo fmt --manifest-path constantine/Cargo.toml -- --check
# cargo clippy --manifest-path constantine/Cargo.toml --target wasm32-unknown-unknown --no-default-features
# cargo build --manifest-path constantine/Cargo.toml --target wasm32-unknown-unknown --no-default-features

cargo clippy --manifest-path zkcrypto/Cargo.toml --all-targets --all-features -- -D warnings
cargo fmt --manifest-path zkcrypto/Cargo.toml -- --check
cargo clippy --manifest-path zkcrypto/Cargo.toml --target wasm32-unknown-unknown --no-default-features
cargo build --manifest-path zkcrypto/Cargo.toml --target wasm32-unknown-unknown --no-default-features
