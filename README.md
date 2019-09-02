# tnetstring

A [TNetString](https://tnetstrings.info) parser for Rust.

## Why?

While there exists a library for parsing TNetStrings in Rust, it doesn't
compile on latest and isn't available through Cargo.

## Prior Art

- <https://github.com/erickt/rust-tnetstring>

## Dev

```shell
cargo test

cargo fmt
cargo clippy -- -Dwarnings

cargo publish
```

## TODO

- serializing to TNetString
- serde support
