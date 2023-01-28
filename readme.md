# Parallelized multi-backend KZG library for Ethereum Data Sharding (aka Proto-Danksharding, EIP-4844)

The goal is to create a parallelized KZG library for Ethereum Data Sharding (aka Proto-Danksharding, EIP-4844) that supports multiple ECC (Elliptic-curve cryptography) backend libraries.

# Backend ECC libraries

Support for multiple backend ECC libraries is implemented via [Traits](https://github.com/sifraitech/kzg/blob/main/kzg/src/lib.rs). Such an approach allows to easy change backend ECC libraries as all the crates shared the same interface (see [benchmarks](https://github.com/sifraitech/kzg/tree/main/kzg-bench/src/benches) and [tests](https://github.com/sifraitech/kzg/tree/main/kzg-bench/src/tests)). The current state of supported backend ECC libraries:

| Backend ECC | FFT/DAS | EIP-4844 (non-parallel) | EIP-4844 (parallel) | [c-kzg-4844](https://github.com/ethereum/c-kzg-4844) drop-in replacement |
| :---: | :---: | :---: | :---: | :---: |
| [blst](https://github.com/supranational/blst) | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| [mcl](https://github.com/herumi/mcl) | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :white_check_mark: |
| [arkworks](https://github.com/arkworks-rs/algebra) | :heavy_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| [zkcrypto](https://github.com/zkcrypto/bls12_381) | :heavy_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |

# Drop-in replacement for c-kzg-4844

We aim to expose [an identical C interface](https://github.com/sifraitech/rust-kzg/blob/b4de1923a6218ea37021d0f9e3bd375dbf529d34/blst-from-scratch/src/eip_4844.rs#L604:L835) compared to [c-kzg-4844](https://github.com/ethereum/c-kzg-4844) so that `rust-kzg` could work as a drop-in replacement for c-kzg-4844. If you already use [c-kzg-4844 bindings](https://github.com/ethereum/c-kzg-4844/tree/main/bindings) you can try faster paralellized `rust-kzg` without any changes to your code-base by simply replacing the binary. Instructions for C#, Java, Nodejs, Python, Rust bindings are available [here](https://github.com/sifraitech/rust-kzg/blob/main/blst-from-scratch/run-c-kzg-4844-tests.sh)

# Example

The best place to look for examples is [tests](https://github.com/sifraitech/kzg/tree/main/kzg-bench/src/tests) directory.

Currently, the ECC backend is set by pointing Cargo to the corresponding crate:

```
[dependencies]
kzg = { git = "https://github.com/sifraitech/kzg.git", package = "blst_from_scratch" }
kzg_traits = { git = "https://github.com/sifraitech/kzg.git", package = "kzg" }
```

# Benchmarks

The latest benchmarks on AMD 3900X:

| Bench | Rust BLST  | Rust BLST parallel | C-KZG (Rust bindings) | Rust MCL   |
| :---: |             :---: |     :---:     |          :---:        | :---: |
| Compute (2 Blobs)  | 140.13 ms | 98.537 ms | 144.48 ms | 143.65 ms |
| Compute (4 Blobs)  | 233.99 ms | 99.359 ms | 246.13 ms | 237.93 ms |
| Compute (8 Blobs)  | 420.68 ms | 105.93 ms | 435.51 ms | 429.48 ms |
| Compute (16 Blobs) | 794.75 ms | 142.87 ms | 827.55 ms | 813.36 ms |
| Verify  (2 Blobs)  | 2.4682 ms | 2.4682 ms | 3.1103 ms | 6.8721 ms |
| Verify  (4 Blobs)  | 3.1072 ms | 3.1072 ms | 4.7926 ms | 11.403 ms |
| Verify  (8 Blobs)  | 4.2477 ms | 4.2477 ms | 7.9565 ms | 20.866 ms |
| Verify  (16 Blobs) | 6.6977 ms | 6.6977 ms | 15.074 ms | 39.335 ms |

Benchmarks [run](https://github.com/sifraitech/kzg/blob/main/.github/workflows/benchmarks.yml) on every Github build. However, it's best to run it on a dedicated machine. [Tautvydas](https://github.com/belijzajac) rendered nice charts for results he got on cloud servers:

![fft fr](images/fft_fr.png)
![fft g1](images/fft_g1.png)
![commit to poly](images/commit_to_poly.png)
![new poly div](images/new_poly_div.png)
![zero poly](images/zero_poly.png)
![das extension](images/das_extension.png)
![recovery](images/recovery.png)

Some results are weird and needs to be double-checked.

# Authors

The project is mainly developed by a group of students at the Blockchain Technologies course led by [Saulius Grigaitis](https://twitter.com/sauliuseth). The project is heavily based on the [go-kzg](https://github.com/protolambda/go-kzg), [c-kzg](https://github.com/benjaminion/c-kzg), and other libraries.
