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
