# MSM Benchmarks

## Description

This crate contains benchmarks for BLS12-381 fixed-base multi-scalar multiplication (MSM) implementations based on BLST.
It benchmarks:

* the BLST `wbits` multiplication routine;
* the `rust-kzg-blst` MSM implementation with a selected algorithm.

The supported algorithms are:

* `wbits` - the window method algorithm;
* `grigaitis_pairwise` - algorithm with pair sum precomputations;
* `grigaitis_pairwise_booth` - pairwise algorithm modification with booth (signed bucket index) encoding;
* `grigaitis_pairwise_cpdlh` - pairwise algorithm modification with booth & additional encoding, as presented in [CPD+24].

## Dataset Information

No static dataset is distributed with this crate. Each benchmark run generates:

* `2^BENCH_NPOW` G1 points;
* `2^BENCH_NPOW` scalars;
* a reference MSM result used for verification.

## Usage Instructions

### Environment setup

Install the following tools before running the benchmarks:

```bash
rustup
cargo
clang
cmake
```

Use the Rust toolchain version pinned by this repository in `rust-toolchain.toml`.

### Ubuntu setup

On Ubuntu, the required system packages can be installed with:

```bash
sudo apt update
sudo apt install -y build-essential clang cmake curl
```

If Rust is not already installed, install `rustup` and the pinned toolchain:

```bash
curl https://sh.rustup.rs -sSf | sh
rustup toolchain install 1.90
rustup default 1.90
```

### Running the benchmarks

To benchmark `wbits`, run:

```bash
BENCH_NPOW=3 WINDOW_SIZE=4 cargo bench -p msm-benches -- "blst wbits mult"
```

To benchmark `grigaitis_pairwise`, run:

```bash
BENCH_NPOW=3 WINDOW_SIZE=4 EXT=1 cargo bench -p msm-benches --features grigaitis_pairwise -- "rust-kzg-blst msm"
```

To benchmark `grigaitis_pairwise_booth`, run:

```bash
BENCH_NPOW=3 WINDOW_SIZE=4 EXT=1 cargo bench -p msm-benches --features grigaitis_pairwise_booth -- "rust-kzg-blst msm"
```

To benchmark `grigaitis_pairwise_cpdlh`, run:

```bash
BENCH_NPOW=3 WINDOW_SIZE=4 EXT=1 cargo bench -p msm-benches --features grigaitis_pairwise_cpdlh -- "rust-kzg-blst msm"
```

### Parameters

Runtime parameters:

* `BENCH_NPOW`: benchmark size exponent; the number of points and scalars is `2^BENCH_NPOW`; default `12`

Compile-time parameters:

* `WINDOW_SIZE`: window size for MSM algorithms;
* `EXT`: extension factor for the `grigaitis_*` algorithms; default `1`

Notes:

* Select only one of `wbits`, `grigaitis_pairwise`, `grigaitis_pairwise_booth`, or `grigaitis_pairwise_cpdlh` at a time.

## Requirements

* Rust 1.90 managed with `rustup`
* Cargo
* a C/C++ toolchain
* `clang`
* `cmake`

On Ubuntu, `build-essential`, `clang`, `cmake`, and `curl` are sufficient for this benchmark crate.

## Methodology

Each benchmark run:

1. generates input points and scalars;
2. computes a reference MSM result, using naive algorithm;
3. constructs the required precomputation table outside the timed benchmark section;
4. measures multiplication time;
5. verifies the benchmark output against the reference result.

## References

[CPD+24]: Yutian Chen, Cong Peng, Yu Dai, Min Luo ir Debiao He. Load-Balanced Parallel Implementation on GPUs for 
Multi-Scalar Multiplication Algorithm. IACR Transactions on Cryptographic Hardware and Embedded Systems, 
2024(2):522–544, 2024.
