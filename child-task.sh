#!/bin/sh

curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain none -y
export PATH="$HOME/.cargo/bin:$PATH"

echo HOME=$HOME
which cargo

cd something-rust
cargo run
