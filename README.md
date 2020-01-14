# conquer-pointer

Strongly typed marked pointers for storing bit patterns alongside raw pointers
for concurrent programming with atomic operations.

[![Build Status](https://travis-ci.org/oliver-giersch/reclaim.svg?branch=master)](
https://travis-ci.org/oliver-giersch/conquer-pointer)
[![Latest version](https://img.shields.io/crates/v/conquer-pointer.svg)](
https://crates.io/crates/conquer-pointer)
[![Documentation](https://docs.rs/conquer-pointer/badge.svg)](https://docs.rs/conquer-pointer)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](
https://github.com/oliver-giersch/conquer-pointer)

## Usage

Add the following to your `Cargo.toml`

```
[dependencies]
conquer-pointer = "0.1.0"
```

## Motivation

Most atomic processor instructions are restricted to only working with
word-sized memory chunks.
Many concurrent lock-free algorithms for data structures require storing
additional data in bitmasks that are composed together with pointers in their
unused bits in order to fit into a single word.

## Future Plans

Before reaching version `1.0.0` the current dependency on the `typenum` crate
will be dropped and replaced with `const_generics`, once these become stable.

## License

Reclaim is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
