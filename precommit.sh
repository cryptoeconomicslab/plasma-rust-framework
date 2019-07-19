#!/bin/sh

cargo fmt --all
cargo clippy --all --all-targets --all-features -- -D warnings
cargo test --all --all-features
