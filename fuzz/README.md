# Running fuzzers

Install cargo-fuzz:

```bash
cargo install cargo-fuzz
```

Then run with command:

```bash
cargo +nightly fuzz run <fuzz-target>
```

Pick any fuzz target from `./fuzz-tragets` directory. You can also specify 
features, to test fuzz MSM algorithms:

```bash
cargo +nightly fuzz run --features wbits <fuzz-target>
```

Available features: `wbits`, `bgmw`, `arkmsm`, `parallel`. Also, you can control
both MSM size with environment variable:

```bash
NPOW=8 cargo +nightly fuzz run <fuzz-target> # this will fuzz with 2^8 points
```

You can also disable some backends from fuzzing (e.g., if some backend has known
limitations):

```bash
cargo +nightly fuzz run --no-default-features <fuzz-target> # fuzz only blst backend msm

cargo +nightly fuzz run --no-default-features --features constantine,arkworks3 <fuzz-target> # fuzz blst, constantine and arkworks3 backends
```

## Some notes on fuzz targets

1. `fixed_msm`/`fixed_msm_with_zeros` targets work faster for precomputation-based MSMs, 
   because bases are fixed so precomputation happens only once.

2. `variable_msm` will be slower, as each time new base needs to be computed, but higher
   coverage can be achieved quicker (as fuzzer can freely explore problem space).
