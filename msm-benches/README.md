# Benchmarking MSMs

This folder contains benchmarks for MSMs:

* [blst wbits algorithm](https://github.com/supranational/blst);
* [crate-crypto's wbits](https://github.com/crate-crypto/rust-eth-kzg/blob/ca7a9e4002c1328abf80ba66838daefaa825dd89/cryptography/bls12_381/src/fixed_base_msm_window.rs#L50);
* all rust-kzg backends.

## Benchmark parameters

You can configure MSM size by specifying environment variable `BENCH_NPOW`. 
Point count will be `2^BENCH_NPOW`. Default is 12 (4096 points). For example:
```bash
BENCH_NPOW=8 cargo bench -p msm-benches # This will benchmark MSMs with 2^8=256 points.
```

You can also select MSMs to benchmark, using criterion's filters. For example:
```bash
cargo bench -p msm-benches -- rust-kzg-blst                      # This will run only rust-kzg-blst MSM
cargo bench -p msm-benches -- "rust-kzg-blst|rust-kzg-arkworks3" # This will run rust-kzg-blst & rust-kzg-arkworks3 MSMs
```

You can also specify seed via `SEED` environment variable. When providing "rand"
value, seed will be selected randomly. For example:
```bash
SEED=rand cargo -p msm-benches # This will generate random set of points & scalars on each run
SEED=7 cargo -p msm-benches    # This will always generate the same set of points & scalars
```

## MSM parameters

### Features

* `--parallel` - run parallel versions of MSMs. Supported by all rust-kzg backends.
* `--bgmw` - use BGMW precomputations. Available for both sequential and parallel mode. Supported by `arkworks4`, `arkworks5`, `blst` and `constantine` backends.
* `--wbits`, `--arkmsm` - use wbits precomputation/arkmsm optimizations. Available only for sequential mode. Supported by `arkworks4`, `arkworks5`, `blst` and `constantine` backends.
