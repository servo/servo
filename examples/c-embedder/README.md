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

## 1. Install the C API

```sh
cargo install cargo-c            # CI pins cargo-c@0.10.23
cargo cinstall -p servo-capi \
    --library-type cdylib \
    --prefix /path/to/servo-sysroot
```

This installs `<prefix>/include/servo/servo_capi.h` and the shared library under
`<prefix>/lib`.

## 2. Build the example

With CMake:

```sh
cmake -S examples/c-embedder -B build -DSERVO_SYSROOT=/path/to/servo-sysroot
cmake --build build
```

Or directly with a C compiler:

```sh
cc -std=c11 -Wall -Wextra examples/c-embedder/main.c \
   -I/path/to/servo-sysroot/include \
   -L/path/to/servo-sysroot/lib \
   -lservo_capi -o servo_embed
```

On Windows with MSVC add `/experimental:c11atomics` (the example uses
`<stdatomic.h>`); clang-cl and MinGW need no extra flag.

## 3. Run it

At runtime the loader must find the shared library:

- Linux: `LD_LIBRARY_PATH=/path/to/servo-sysroot/lib ./servo_embed`
- macOS: `DYLD_LIBRARY_PATH=/path/to/servo-sysroot/lib ./servo_embed`
- Windows: keep `servo_capi.dll` (and the ANGLE `libEGL.dll` / `libGLESv2.dll`)
  next to `servo_embed.exe`. The CMake build copies the DLLs for you.

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
