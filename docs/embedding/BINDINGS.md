# Writing a language binding for the Servo C API

[`C_API_GUIDE.md`](C_API_GUIDE.md) explains the API to people writing C. This
document is the shorter, stricter list for people wrapping it in Python, Java,
.NET, Go, or anything else with a GC and its own scheduler: the rules the C ABI
cannot enforce for you, and the mistakes that only show up under collection,
a second thread, or a version skew.

**Scope.** This branch ships the C API, a C++ RAII wrapper (exercised by
`tests/capi/cpp/test_cpp_api.cpp`), and a standalone sample consumer
(`examples/c-embedder`, built in CI against an installed sysroot). It ships no
Python, Java, or .NET binding — see [Why bindings stay out of tree](#why-bindings-stay-out-of-tree).

## 1. One thread owns Servo

Servo is single-threaded from the embedder's point of view. Create the
`Servo`, build `WebView`s, and call `servo_spin_event_loop` from **one**
thread. Almost everything in the C API must be called from it, including from
inside callbacks that came from it.

Three callbacks are exceptions, and a binding that assumes one rule for all of
them will be wrong somewhere:

| Callback | Thread it may arrive on |
|---|---|
| `ServoEventLoopWaker.wake_callback` | **Any** Servo-internal thread |
| `ServoProtocolHandler` request callbacks (`intercept_request`, `intercept_request_streaming`) | **Network or background** threads |
| Everything in `ServoWebViewDelegate` | The embedder thread, inside `servo_spin_event_loop` |

What follows for a binding:

- **There is no "call Servo from anywhere" primitive.** The API exports no
  wake function: the waker only goes from Servo to you. If another host thread
  needs to do something with Servo, it must notify your loop (a queue, a
  condition variable, a GIL-agnostic event) and let the owning thread make the
  call.
- **A GC finalizer must never touch a `Servo` handle.** Finalizers and
  destructors run on the GC's thread. Queue the free to your loop thread
  instead, or keep the handles alive until the loop is torn down.
- **The waker must stay trivial.** It may not call any Servo function, must not
  block, and must be safe from any thread. Its only job is to wake your loop so
  that *your* thread calls `servo_spin_event_loop`.
- **Protocol-handler callbacks are the trap.** They run off the embedder thread,
  so a binding must not assume thread-local storage, a lock the embedder thread
  already holds, or a non-reentrant interpreter lock. Copy what you need and
  return.

## 2. Fill-structs: always set `struct_size`

Every struct passed *into* Servo (`ServoDelegate`, `ServoWebViewDelegate`,
`ServoEventLoopWaker`, ...) starts with a `struct_size` field. Set it to
`sizeof` the struct **as your binding saw it**, and zero-initialise the rest.

The library accepts a struct only when `struct_size >=` the size it was compiled
with. That has a consequence bindings usually get backwards:

- **Your header must be at least as new as the library you load.** A binding
  compiled against an older header, loaded against a newer `servo_capi`, passes
  a smaller `struct_size` and gets `SERVO_ERROR_INVALID_ARGUMENT` — a clean
  error, but one that looks like a random failure if the binding never checks it.
- A binding must therefore negotiate versions **at load time**, not assume that
  "it linked" means "it is compatible". Call `servo_capi_version()` after
  loading and compare with `SERVO_CAPI_VERSION` from the header you generated;
  refuse to proceed on a mismatch outside your supported range.

Designated initialisers (`.field = value`) keep this honest in C/C++: a field
that did not exist when you compiled simply does not participate.

## 3. Borrowed strings and buffers

Almost every `const char*` you receive is **valid only for the duration of the
callback** — `url`, `title`, `message`, `result_json`, `error_message`,
screenshot `data`, custom-scheme request fields. Copy it if you need to keep it;
none of these are freed by anyone, and reading one after the callback returns is
undefined behaviour.

The buffer-query pattern is the mirror image: `servo_webview_get_url(webview,
NULL, 0)` returns the length (including the NUL), you allocate, then call again.
A binding should expose that as "returns a string" and hide the two calls — but
must keep the ownership rule, i.e. the buffer is the *caller's* and is freed
with `free()`, not by Servo.

## 4. Ownership

Two distinct classes, and mixing them is a double free or a leak:

- **Ownership transfers.** `servo_builder_set_options` consumes `ServoOptions`;
  `servo_webview_builder_create` consumes the `RenderingContext`;
  `servo_builder_build` consumes the builder. After a transfer the handle is
  dead — do not free it, do not use it.
- **`*_free` functions.** Everything you created and did not transfer is freed
  by its own `servo_*_free`. Order matters: WebViews before `Servo`,
  `Servo` before anything it owns.
- **Buffers the API gives you back.** `char*` from
  `servo_preferences_get_string` is caller-owned and must be released with the
  C `free()` that matches the allocator that produced it. On Windows, verify
  `servo_capi` and your binding agree on a C runtime before freeing across the
  boundary — copy the bytes and free immediately if you cannot.
- **`ServoValue` handles.** In the `servo_webview_evaluate_javascript_v2`
  callback, a non-`NULL` `result_value` is *your* handle and must be released
  with `servo_value_free`.

The full table lives in
[`C_API_GUIDE.md#ownership-and-free-rules`](C_API_GUIDE.md#ownership-and-free-rules).
A binding should encode it as ownership in its wrapper types rather than
documenting it for users to remember.

## 5. One-shot callbacks

Several callbacks carry an opaque `user_data` **plus** a response function that
consumes a request (`ServoColorPickerCallback`, `ServoSelectElementCallback`,
`ServoClipboardStringRequest` resolution, custom-scheme responses, ...).

- The `user_data` you are handed there is *an opaque handle Rust gave you*, not
  your delegate's `user_data`. Pass it back to the response function unchanged.
- **Exactly one response per request.** The request is a `Once`. Never calling
  it leaves the page waiting for a control that never resolves, and leaks the
  responder.
- A `NULL` field in a delegate is a deliberate answer, not "I will do it later":
  Servo resolves the control itself with the non-destructive default. This is
  how a binding should express "not implemented".

## 6. Errors and panics

Every fallible entry point returns `ServoError` (`0` success, negative failure).
The boundary is `catch_unwind`-protected: no Rust panic crosses into your
language, it comes back as `SERVO_ERROR_INTERNAL_PANIC` (`-8`) or `NULL`. Your
binding should surface those codes instead of treating a `NULL` as a generic
exception — and should not assume a negative value means "throw", because some
callers want to handle `SERVO_ERROR_INVALID_STATE` as a normal flow.

`servo_capi_version()` returns static storage that must never be freed.

## 7. Building the binding

- Generate or vendor the header: it is produced by `cbindgen` from
  `ffi/capi/cbindgen.toml`, and CI checks it against `ffi/capi/symbols.txt`.
  **Never hand-edit it**, and never re-declare the API in your own file — the
  header is the contract, the symbol allowlist is what the library actually
  exports.
- Prefer the packaged sysroot over building from source: `support/pkgconfig/servo_capi.pc`
  and `support/cmake/ServoConfig.cmake` are installed by the artifact workflow, which then
  builds `examples/c-embedder` twice - once through CMake `find_package` and once by
  calling the compiler directly against the sysroot - on Linux, macOS, and Windows.
  `examples/c-embedder/README.md` documents the layout.
- Load the library lazily and resolve the version first (§2), then symbols.
  Symbols are append-only; anything `ffi/capi/symbols.txt` does not list is not
  part of the contract.
- Your wrapper should own, not expose: one object per handle, a single place
  that calls `*_free`, and a hard error if a handle is used after its owner was
  released.

## Language notes

**Python.** The GIL makes §1 concrete: every Servo callback arrives on the loop
thread, so the binding must call into Python there with the interpreter attached,
and must never let Servo's threads call into Python unattached. Do not expose
Servo calls on arbitrary Python threads — route them to the loop. Avoid
`ctypes`/`cffi` callbacks that re-enter Servo while holding the loop inside
another call; the API is not reentrant from a callback except where the guide
says so.

**Java / JNI.** Same shape as Python: attach/detach is per-thread and must be
handled for every foreign thread that reaches a callback. Prefer global refs for
anything that outlives the call, and never `DeleteGlobalRef` from a Servo
thread without having attached first.

**.NET.** Delegates passed to Servo must be kept alive for the lifetime of the
handle they were installed on — the C API copies function pointers, not
managed objects. Pin or root them for the WebView's lifetime, and marshal work
back to the loop thread.

**Rust.** If you are writing Rust, do not use the C API: the in-tree `servo`
crate (`components/servo`) is the native API and is what `tests/capi` binds
through `servo_api`. The C API exists for non-Rust hosts.

**C++.** Use the existing RAII wrapper pattern from
`tests/capi/cpp/test_cpp_api.cpp` as the reference: it covers header
consumption under C++17, ownership wrappers, and designated initialisers for
delegates.

## Why bindings stay out of tree

- **Release cadence.** A binding pins a header and a symbol list; the engine
  moves faster. In-tree it would either block engine changes or rot.
- **Dependency choices.** Each language has its own packaging, CI matrix, and
  runtime story; none of them belong in Servo's build.
- **The contract is what matters.** The C API is append-only and versioned
  (`ABI_STABILITY.md`). A binding that follows this document should not need to
  track Servo's internals, only the version range it declares.

So: out of tree, but **supported** — "it is not in this repository" is not the
same as "it will break on you".

## Checklist before publishing a binding

1. Loads `servo_capi`, checks `servo_capi_version()` against the header it was
   generated from, and refuses a version outside its declared range.
2. Sets `struct_size` on every struct it passes in.
3. Copies every borrowed string/buffer before the callback returns.
4. Frees every handle it owns exactly once, in dependency order; has no
   Servo call from a finalizer or a foreign thread.
5. Answers every one-shot callback exactly once, or leaves the delegate field
   `NULL` on purpose.
6. Handles `SERVO_ERROR_INTERNAL_PANIC` as a first-class failure.
7. Documents its supported version range and where its header comes from.
