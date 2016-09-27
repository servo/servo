# Geckolib tools

This directory contains simple tools for generating the Rust bindings for [stylo](https://public.etherpad-mozilla.org/p/stylo).

## `setup_bindgen.sh`

This clones Servo's version of bindgen, and uses `llvm-3.8` library to build it. It will then be used to generate the Rust bindings.

## `regen.sh`

This will regenerate the bindings for the `ServoBindings.h` file in your gecko
build. The generated bindings live in `components/style/gecko_bindings/bindings.rs`. For structs, the bindings are in `components/style/gecko_bindings/structs_*`
