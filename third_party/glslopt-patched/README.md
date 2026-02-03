# glslopt-patched

This is a vendored copy of [glslopt-rs](https://github.com/jamienicol/glslopt-rs) with a fix for Windows ARM64 (aarch64-pc-windows-msvc) compilation.

## Why this exists

On Windows ARM64, the MSVC compiler includes `<arm_fp16.h>` which defines:
```c
typedef __fp16 float16_t;
```

This conflicts with `struct float16_t` defined in `glsl-optimizer/src/util/half_float.h`, causing a compilation error.

The fix renames the struct to `mesa_float16_t` to avoid the collision.

## Upstream PR

This fix has been submitted upstream: https://github.com/jamienicol/glslopt-rs/pull/11

## Removal

Once the upstream PR is merged and a new version of glslopt-rs is released, this vendored copy can be removed by:

1. Removing the `[patch.crates-io]` section for glslopt in the root `Cargo.toml`
2. Deleting this `third_party/glslopt-patched/` directory
3. Running `cargo update -p glslopt`
