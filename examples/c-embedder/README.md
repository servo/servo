# servo-capi C embedder example

A minimal, standalone C11 program that embeds Servo through the `servo-capi`
shared library and its generated C header. It is the runnable counterpart to the
self-contained example in [`docs/embedding/C_API_GUIDE.md`](../../docs/embedding/C_API_GUIDE.md).

Unlike the C tests under [`tests/capi`](../../tests/capi), this example does **not**
depend on the Servo source tree. It links only against an *installed* sysroot,
which is the contract a third-party embedder actually uses.

## What it does

`main.c`:

1. builds a single-process, unsandboxed `Servo` with a software rendering
   context (so it runs headless on every platform);
2. opens a `WebView` with a delegate, loading a `data:` URL by default;
3. spins the event loop manually, painting on new frames;
4. evaluates `document.title` asynchronously and prints the result;
5. takes a screenshot and prints the first pixel;
6. queries the final URL and title with the buffer-query pattern;
7. tears everything down in reverse order.

## 1. Install the C API into a sysroot

Build and install `servo-capi` using `cargo-c`:

```sh
cargo install cargo-c            # CI pins cargo-c@0.10.23
cargo cinstall -p servo-capi \
    --library-type cdylib \
    --release \
    --destdir /path/to/servo-sysroot \
    --prefix /
```

Copy the CMake and pkg-config integration files into the sysroot:

```sh
# CMake package configuration
mkdir -p /path/to/servo-sysroot/lib/cmake/Servo
cp support/cmake/ServoConfig.cmake /path/to/servo-sysroot/lib/cmake/Servo/
cp support/cmake/ServoConfigVersion.cmake /path/to/servo-sysroot/lib/cmake/Servo/

# pkg-config definition
mkdir -p /path/to/servo-sysroot/lib/pkgconfig
cp support/pkgconfig/servo_capi.pc /path/to/servo-sysroot/lib/pkgconfig/
```

This populates:
- `<sysroot>/include/servo/servo_capi.h`
- `<sysroot>/lib/` (shared libraries: `.so` on Linux, `.dylib` on macOS, `.dll.a` and `bin/*.dll` on Windows)
- `<sysroot>/lib/cmake/Servo/ServoConfig.cmake`
- `<sysroot>/lib/pkgconfig/servo_capi.pc`

## 2. Build the example locally

### Option A: CMake (Recommended — uses `find_package(Servo CONFIG)`)

```sh
cmake -S examples/c-embedder -B build -DCMAKE_PREFIX_PATH=/path/to/servo-sysroot
cmake --build build
```

On Windows with MinGW Makefiles:
```sh
cmake -S examples/c-embedder -B build -G "MinGW Makefiles" \
    -DCMAKE_PREFIX_PATH=/path/to/servo-sysroot \
    -DCMAKE_C_COMPILER=gcc
cmake --build build
```

### Option B: Direct compilation with `pkg-config`

```sh
export PKG_CONFIG_PATH=/path/to/servo-sysroot/lib/pkgconfig:${PKG_CONFIG_PATH:-}
gcc -std=c11 -Wall -Wextra examples/c-embedder/main.c \
    $(pkg-config --cflags servo_capi) \
    $(pkg-config --libs servo_capi) \
    -o servo_embed
```

### Option C: Direct compiler invocation

**GCC / Clang (Linux, macOS, MinGW on Windows):**
```sh
gcc -std=c11 -Wall -Wextra examples/c-embedder/main.c \
    -I/path/to/servo-sysroot/include \
    -L/path/to/servo-sysroot/lib \
    -lservo_capi -o servo_embed
```

**MSVC on Windows:**
```cmd
cl /std:c11 /experimental:c11atomics /W4 examples\c-embedder\main.c ^
   /I /path/to/servo-sysroot\include ^
   /link /LIBPATH:/path/to/servo-sysroot\lib servo_capi.lib /out:servo_embed.exe
copy \path\to\servo-sysroot\bin\servo_capi.dll .
```

## 3. Run it

At runtime the loader must locate the shared library:

- **Linux:** `LD_LIBRARY_PATH=/path/to/servo-sysroot/lib ./build/servo_embed`
- **macOS:** `DYLD_LIBRARY_PATH=/path/to/servo-sysroot/lib ./build/servo_embed`
- **Windows:** keep `servo_capi.dll` next to `servo_embed.exe` (CMake copies it automatically during build).

```sh
./servo_embed                       # loads the built-in data: URL
./servo_embed https://example.com/  # loads a real page
```

## Notes

- **One `Servo` per process.** Servo's embedding runtime does process-global,
  one-shot initialization (JS engine, async runtime, logger, options). Create a
  single `Servo` and share it.
- **Single-threaded from the embedder's point of view.** All API calls and all
  delegate callbacks happen on the thread that calls `servo_spin_event_loop`.
  The waker is the only exception and may run on any Servo thread — it must not
  touch Servo handles.
- **Callbacks hand out temporary pointers.** Strings and byte buffers passed to
  callbacks are valid only for the duration of the call; copy them to keep them.
- **No panic ever crosses the FFI.** Fallible entry points return an error code
  or `NULL`; the shared library catches Rust panics at the boundary.

See [`docs/embedding/C_API_GUIDE.md`](../../docs/embedding/C_API_GUIDE.md) for the
full API surface, and
[`docs/embedding/ABI_STABILITY.md`](../../docs/embedding/ABI_STABILITY.md) for the
compatibility policy.
