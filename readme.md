# Parallelized multi-backend KZG library for Ethereum Data Sharding (aka Proto-Danksharding, EIP-4844)

The goal is to create a parallelized KZG library for Ethereum Data Sharding (aka Proto-Danksharding, EIP-4844) that supports multiple ECC (Elliptic-curve cryptography) backend libraries.

# Backend ECC libraries

Support for multiple backend ECC libraries is implemented via [Traits](https://github.com/sifraitech/kzg/blob/main/kzg/src/lib.rs). Such an approach allows to easily change backend ECC libraries in a way that is done in [benchmarks](https://github.com/sifraitech/kzg/tree/main/kzg-bench/src/benches) and [tests](https://github.com/sifraitech/kzg/tree/main/kzg-bench/src/tests). List of different backend ECC libraries:

* Arkworks - Rust implementation using [akrworks](https://github.com/arkworks-rs);
* blst-from-scratch - Rust implementation using [blst](https://github.com/supranational/blst);
* ckzg - bindings for [c-kzg](https://github.com/benjaminion/c-kzg) that uses [blst](https://github.com/supranational/blst);
* mcl - Rust implementation using [mcl](https://github.com/herumi/mcl);
* zkcrypto - Rust implementation using [zkcrypto](https://github.com/zkcrypto).

# Example

The best place to look for examples is [tests](https://github.com/sifraitech/kzg/tree/main/kzg-bench/src/tests) directory.

```
[dependencies]
kzg = { git = "https://github.com/sifraitech/kzg.git", package = "blst_from_scratch" }
kzg_traits = { git = "https://github.com/sifraitech/kzg.git", package = "kzg" }
```

```
use kzg::eip_4844::{bytes_to_bls_field, compute_powers};
use kzg_traits::*;

const EXPECTED_POWERS: [[u64; 4usize]; 11] = [
    [1, 0, 0, 0],
    [32930439, 0, 0, 0],
    [1084413812732721, 0, 0, 0],
    [15773128324309817559, 1935, 0, 0],
    [17639716667354648417, 63748557064, 0, 0],
    [14688837229838358055, 2099267969765560859, 0, 0],
    [17806839894568993937, 15217493595388594120, 3747534, 0],
    [17407663719861420663, 10645919139951883969, 123407966953127, 0],
    [9882663619548185281, 9079722283539367550, 5594831647882181930, 220],
    [4160126872399834567, 5941227867469556516, 11658769961926678707, 7254684264],
    [4000187329613806065, 4317886535621327299, 17988956659770583631, 238899937640724696]
];

fn main() {
    let x: u64 = 32930439;
    let n = 11;

    let x_bytes: [u8; 32] = u64_to_bytes(x);

    let x_bls = bytes_to_bls_field(&x_bytes);

    let powers = compute_powers(&x_bls, n);

    for (p, expected_p) in powers.iter().zip(EXPECTED_POWERS.iter()) {
        assert_eq!(expected_p, &p.to_u64_arr());
    }
}

fn u64_to_bytes(x: u64) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[0..8].copy_from_slice(&x.to_le_bytes());
    bytes
}
```

# Benchmarks

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
