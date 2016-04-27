# GeckoLib tools

This directory contains mostly simple tools for working with
[stylo](https://public.etherpad-mozilla.org/p/stylo).

Some scripts require [multirust](https://github.com/brson/multirust) in order to
work.

You can see a description of them below.

## `setup_bindgen.sh`

This uses downloads a custom version of bindgen, up to date to generate the
bindings, and uses the custom `clang` to build it.

It will also rebuild it if it's already downloaded.

## `regen_bindings.sh`

This will regenerate the bindings for the `ServoBindings.h` file in your gecko
build (which are in `ports/geckolib/bindings.rs`).

## `regen_style_structs.sh`

This will generate the bindings for Gecko's style structs. Current bindings are
actually in `ports/geckolib/gecko_style_structs.rs`.
