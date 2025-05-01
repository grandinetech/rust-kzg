# Benchmarking MSMs

This folder contains benchmarks for MSMs:

* [blst wbits algorithm](https://github.com/supranational/blst);
* [crate-crypto's wbits](https://github.com/crate-crypto/rust-eth-kzg/blob/ca7a9e4002c1328abf80ba66838daefaa825dd89/cryptography/bls12_381/src/fixed_base_msm_window.rs#L50);
* all rust-kzg backends.

## Benchmark parameters

You can configure MSM size by specifying environment variable `BENCH_NPOW`. 
Point count will be `2^BENCH_NPOW`. Default is 12 (4096 points). For example:
```bash
BENCH_NPOW=8 cargo -p msm-benches bench # This will benchmark MSMs with 2^8=256 points.
```

You can also select MSMs to benchmark, using criterion's filters. For example:
```bash
cargo bench -p msm-benches -- rust-kzg-blst                      # This will run only rust-kzg-blst MSM
cargo bench -p msm-benches -- "rust-kzg-blst|rust-kzg-arkworks3" # This will run rust-kzg-blst & rust-kzg-arkworks3 MSMs
```

## MSM parameters

### Features

* `--parallel` - run parallel versions of MSMs. Supported by all rust-kzg backends.
* `--bgmw` - use BGMW precomputations. Available for both sequential and parallel mode. Supported by `arkworks4`, `arkworks5`, `blst` and `constantine` backends.
* `--wbits`, `--arkmsm` - use wbits precomputation/arkmsm optimizations. Available only for sequential mode. Supported by `arkworks4`, `arkworks5`, `blst` and `constantine` backends.

### Environment variables

* `WINDOW_SIZE` - window length for pippenger, BGMW, arkmsm and wbits algorithms.
* `WINDOW_NX` - amount of points processed by one thread. Used only in parallel versions for pippenger and BGMW.

Here is table for default WINDOW_SIZE parameter value, when running MSMs in sequential mode:

| Point count | 2^6 | 2^7 | 2^8 | 2^9 | 2^10 | 2^11 | 2^12 | 2^13 | 2^14 |
|-------------|-----|-----|-----|-----|------|------|------|------|------|
| Pippenger   | 4   | 5   | 6   | 7   | 8    | 9    | 10   | 10   | 11   |
| BGMW        | 8   | 9   | 10  | 10  | 11   | 12   | 13   | 13   | 15   |
| Arkmsm      | 8   | 8   | 8   | 8   | 10   | 10   | 10   | 12   | 12   |
| WBITS       | 8   | 8   | 8   | 8   | 8    | 8    | 8    | 8    | 8    |

Here is table for default WINDOW_SIZE + WINDOW_NX parameter values, when running MSMs in parallel mode, 4 cores:

| Point count | 2^6       | 2^7       | 2^8       | 2^9       | 2^10       | 2^11       | 2^12       | 2^13       | 2^14       |
|-------------|-----------|-----------|-----------|-----------|------------|------------|------------|------------|------------|
| Pippenger   | nx=1, w=5 | nx=1, w=6 | nx=1, w=7 | nx=1, w=8 | nx=1, w=9  | nx=1, w=9  | nx=1, w=11 | nx=1, w=11 | nx=1, w=11 |
| BGMW        | nx=4, w=7 | nx=1, w=8 | nx=1, w=8 | nx=4, w=9 | nx=4, w=10 | nx=1, w=11 | nx=1, w=11 | nx=1, w=13 | nx=1, w=13 |

8 cores:
| Point count | 2^6       | 2^7       | 2^8       | 2^9       | 2^10       | 2^11       | 2^12       | 2^13       | 2^14       |
|-------------|-----------|-----------|-----------|-----------|------------|------------|------------|------------|------------|
| Pippenger   | nx=1, w=5 | nx=1, w=6 | nx=1, w=7 | nx=1, w=8 | nx=1, w=8  | nx=1, w=9  | nx=1, w=11 | nx=1, w=11 | nx=1, w=11 |
| BGMW        | nx=8, w=6 | nx=8, w=7 | nx=1, w=8 | nx=1, w=8 | nx=8, w=9  | nx=8, w=10 | nx=1, w=11 | nx=1, w=11 | nx=8, w=12 |

16 cores:
| Point count | 2^6        | 2^7        | 2^8        | 2^9       | 2^10       | 2^11       | 2^12        | 2^13        | 2^14        |
|-------------|------------|------------|------------|-----------|------------|------------|-------------|-------------|-------------|
| Pippenger   | nx=1, w=4  | nx=1, w=6  | nx=1, w=6  | nx=1, w=8 | nx=1, w=8  | nx=1, w=9  | nx=1, w=10  | nx=1, w=10  | nx=1, w=11  |
| BGMW        | nx=16, w=5 | nx=16, w=6 | nx=16, w=7 | nx=1, w=8 | nx=1, w=8  | nx=16, w=9 | nx=16, w=10 | nx=16, w=10 | nx=16, w=11 | 

32 cores:
| Point count | 2^6        | 2^7        | 2^8        | 2^9        | 2^10       | 2^11       | 2^12       | 2^13        | 2^14        |
|-------------|------------|------------|------------|------------|------------|------------|------------|-------------|-------------|
| Pippenger   | nx=1, w=4  | nx=1, w=5  | nx=1, w=6  | nx=1, w=8  | nx=1, w=7  | nx=1, w=8  | nx=1, w=9  | nx=1, w=9   | nx=1, w=10  | 
| BGMW        | nx=32, w=5 | nx=32, w=5 | nx=32, w=6 | nx=32, w=7 | nx=1, w=8  | nx=1, w=8  | nx=32, w=9 | nx=32, w=10 | nx=32, w=10 |
