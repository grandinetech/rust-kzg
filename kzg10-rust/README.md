## Improve KZG10-rust
### Team:
- Arnas Rimkus
- Jaroslav Kochanovskis
- Gytis Becalis

### Summary:
As an assignment for Blockchain Technology course we have to improve [kzg10-rust](https://github.com/UndeadRat22/kzg10-rust) library by porting tests and benchmarks from the [c-kzg](https://github.com/benjaminion/c-kzg) library.

#### Additional sources (our):
- [Tests Excel](https://docs.google.com/spreadsheets/d/1pp2ou4aIsSjNDQ8XuvN6Dh0LmMSiK4ekxlNIHFfE3kY/edit?usp=sharing);


# KZG10 (Kate) Polynomial Commitments

## About

This is a KZG10 implmentation in Rust, heavily based on the Go implementation by protolambda https://github.com/protolambda/go-kzg.

This repo also contains Herumi MCL, as I could not find a way to make the cargo crate for mcl_rust work, and it does seem abandoned. It can be found here: https://github.com/herumi/mcl-rust.

## How to test/run

First follow the steps in Herumi mcl (refer to the link above), then just run the following code in checked out dir:

```bash
cargo test -- --test-threads 1 --nocapture
```

To run benchmarks, just run

```bash
cargo bench
```
