# servo-capi-tests

This crate hosts the unit and integration tests for the C API exposed by the `servo-capi` crate.
`cargo` currently doesn't allow dependending on a cdylib crate without enabling unstable features.
Since we want to test the shared object and not the statically linked rlib, we use this configuration where we compile and link the `servo-capi` crate manually in `build.rs` using `cargo-c`.

The tests are still triggered using `cargo test` by exposing the entry point to the tests from the C test runner and invoking them from Rust using `extern fn`.

### Running the tests
Run ` cargo test -p servo-capi-tests` from the root to exectute the tests.
