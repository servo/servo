# servo-capi

This crate provides a C FFI wrapper for Servo's native Rust API. 

The goal is to allow Servo to be consumed by embedders in other languages and eventually also provide a stable ABI for Servo by wrapping this crate again in a Rust API.

### Public headers
`cbindgen.toml` generates `servo_capi.h`, the C header. It is generated with
`cpp_compat = true`, so it can also be `#include`d from C++. `servo_capi.hpp` is a
hand-written, header-only C++17 RAII wrapper over that C API; it is compiled by the
C++ tests in `tests/capi/cpp`. See `docs/embedding/C_API_GUIDE.md` for usage.

### Testing
The tests for the C API exposed by this crate live in the `servo-capi-tests` crate that lives in `tests/capi`.
Refer to that crate's README.md for the rationale and how to run the tests.
