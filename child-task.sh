#!/bin/sh

curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain none -y

cd something-rust
rustup run --install cargo run
