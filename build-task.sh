#!/bin/sh

set -e
set -x

cd something-rust
cargo build --release
gzip -c target/release/something-rust > executable.gz
