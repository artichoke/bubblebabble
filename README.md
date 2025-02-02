# boba

[![GitHub Actions](https://github.com/artichoke/boba/workflows/CI/badge.svg)](https://github.com/artichoke/boba/actions)
[![Code Coverage](https://codecov.artichokeruby.org/boba/badges/flat.svg?nocache=2)](https://codecov.artichokeruby.org/boba/index.html)
[![Discord](https://img.shields.io/discord/607683947496734760)](https://discord.gg/QCe2tp2)
[![Twitter](https://img.shields.io/twitter/follow/artichokeruby?label=Follow&style=social)](https://twitter.com/artichokeruby)
<br>
[![Crate](https://img.shields.io/crates/v/boba.svg)](https://crates.io/crates/boba)
[![API](https://docs.rs/boba/badge.svg)](https://docs.rs/boba)
[![API trunk](https://img.shields.io/badge/docs-trunk-blue.svg)](https://artichoke.github.io/boba/boba/)

Implements the the [Bubble Babble binary data encoding][bubble-babble-spec].

> The Bubble Babble Encoding encodes arbitrary binary data into pseudowords that
> are more natural to humans and that can be pronounced relatively easily.

Bubble Babble encodes 6 characters in 16 bits and includes a checksum embedded
in the encoded data. See the [Bubble Babble spec][bubble-babble-spec].

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
boba = "6.0.0"
```

Then encode and decode data like:

```rust
assert_eq!(boba::encode("Pineapple"), "xigak-nyryk-humil-bosek-sonax");
assert_eq!(boba::decode(b"xexax"), Ok(vec![]));
```

## Crate Features

Boba is `no_std` compatible with a required dependency on the [`alloc`] crate.

`boba` is [fuzzed](fuzz/fuzz_targets) with [cargo-fuzz].

## Minimum Rust Version Policy

This crate's minimum supported `rustc` version (MSRV) is `1.83.0`.

MSRV may be bumped in minor version releases.

## License

`boba` is licensed under the [MIT License](LICENSE) (c) Ryan Lopopolo.

[bubble-babble-spec]: spec/Bubble_Babble_Encoding.txt
[`alloc`]: https://doc.rust-lang.org/stable/alloc/index.html
[`std`]: https://doc.rust-lang.org/stable/std/index.html
[`std::error::error`]:
  https://doc.rust-lang.org/stable/std/error/trait.Error.html
[cargo-fuzz]: https://crates.io/crates/cargo-fuzz
