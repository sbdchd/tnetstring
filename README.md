# tnetstring [![cargo-badge](https://img.shields.io/crates/v/tnetstring.svg)](https://crates.io/crates/tnetstring) [![CircleCI](https://circleci.com/gh/sbdchd/tnetstring.svg?style=svg)](https://circleci.com/gh/sbdchd/tnetstring)

A [TNetString](https://tnetstrings.info) serde plugin and parser for Rust.

## Install

```shell
cargo add tnetstring
```

## Why?

While there exists a library for parsing TNetStrings in Rust, it doesn't
compile on latest and isn't available through Cargo. It also lacks serde
support.

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

- serde support for `f32` and `f64`
