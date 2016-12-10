# Geckolib tools

This directory contains simple tools for generating the Rust bindings for [stylo](https://public.etherpad-mozilla.org/p/stylo).

## `setup_bindgen.sh`

This clones Servo's version of bindgen, and uses `llvm-3.8` library to build it. It will then be used to generate the Rust bindings.
