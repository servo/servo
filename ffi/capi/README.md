# servo-capi

This crate provides a C FFI wrappers for  Servo's native Rust API. 

The goal is to allow Servo to be consumed by embedders in other languages and
eventually also provide a stable ABI for Servo by wrapping this crate again in
a Rust API.

### Testing
The tests for the C API exposed by this crate live in the `servo-capi-tests` crate. 
Refer to that crate's README.md for the rationale and how to run the tests.
