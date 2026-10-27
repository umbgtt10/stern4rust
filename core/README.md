# stern4rust-core

The rules behind [`cargo-stern4rust`](https://crates.io/crates/cargo-stern4rust):
a Rust workspace held to its house coding rules, every file that breaks one
named.

This crate is the technology-free half of the tool. It is `#![no_std]` and
reaches the file system, `cargo metadata`, TOML, JSON and stdout only through
the ports in `stern4rust::traits`, which the command-line crate implements.
It is published so that the command can depend on it; it is not a public API,
and its paths and signatures may move in any release. Depend on the binary,
`cargo stern4rust`.

See the [repository README](https://github.com/umbgtt10/stern4rust) for what
the rules are and how to run them.
