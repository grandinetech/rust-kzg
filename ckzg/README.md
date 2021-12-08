# Dependencies

## Linux (Ubuntu/Debian)

- libomp-dev

install it by typing `apt install libomp-dev`

to find out the path to `libomp`, execute the following command:

```bash
find /usr/lib/llvm* -name libiomp5.so | head -n 1
```

## MacOS

- libomp
- gnu-sed

install them by typing `brew install libomp gnu-sed`

the path to `libomp` is `/usr/local/lib/libomp.dylib`

# Building native libs

```bash
bash build.sh
```

# Compiling Rust code

the `RUSTFLAGS` environment variable pointing to the `libomp` LLVM runtime needs to be set

to do so, you can either export `RUSTFLAGS=-C link-arg=PATH_TO_LIBOMP` or create  a `.cargo/config.toml` file in the `ckzg` folder with the following content:

```toml
[build]
rustflags = [
  "-C", "link-arg=PATH_TO_LIBOMP"
]
```
