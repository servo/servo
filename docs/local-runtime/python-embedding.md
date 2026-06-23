# Python Embedding API

Severin has an experimental CPython native extension crate at `ports/severin-python/`. The extension module is named `severin` and embeds the host-side Servo instance directly in the Python process. It must not spawn a `severin` executable, bind a port, or communicate through HTTP, WebSocket, Unix socket, or localhost IPC.

## Initial API Shape

```python
import severin

app = severin.App(width=800, height=600, bridge=None)
app.load_path("app/index.html")
app.run()
frame = app.read()
if frame is not None:
    receipt, json_text = frame
    app.write(receipt, '{"ok": true}')
app.close()
```

The Rust object behind `severin.App` owns:

- a `servo::Servo` instance built with `ServoBuilder`
- a `servo::WebView` attached to that Servo instance
- a software rendering context sized from the requested `width` and `height`
- the optional Python `bridge` object, retained only as future host-side state
- a native transport queue for opaque serialized JSON bridge frames

`App.load_path(path: str)` currently treats `path` as the package entry file. It sets the existing first-milestone package environment (`SERVORENA_PACKAGE_ID=com.example.app`, `SERVORENA_PACKAGE_ROOT=<entry parent>`) and navigates the embedded WebView to `asset://com.example.app/<entry filename>`. This is deliberately an adapter over the current package wall, not the final multi-package provider API.

`App.run()` spins Servo's in-process event loop and returns when the current load reaches `LoadStatus::Complete` or after `App.close()` has marked the instance closed. `App.close()` drops the owned Servo/WebView state in-process so Servo's normal drop-time shutdown path can run.

## Bridge Transport Model

The bridge is a mail slot with numbered receipts, not an application protocol. The native layer must not define action names, capability names, permission rules, success/error conventions, request schemas, reply schemas, or a registry of host functions. It must not interpret application payload contents beyond the minimum needed to carry valid JSON.

The intended flow is:

1. JavaScript submits arbitrary JSON through a tiny shim and receives a Promise.
2. The JS shim records the Promise in its own pending table using private transport bookkeeping.
3. Rust enqueues the serialized JSON frame and an opaque receipt.
4. Python calls `App.read()` and receives either `None` or `(receipt, json_text)`.
5. Python decides whether to ignore, defer, sort, reject, answer, or otherwise act on the JSON text.
6. Python calls `App.write(receipt, json_text)` with arbitrary valid JSON for that receipt.
7. The JS shim resolves the matching Promise with that JSON verbatim.

The receipt is not part of the JSON protocol. It is a private transport handle used only to route a reply to the pending Promise that caused the inbound frame. Application code may choose any JSON convention it wants inside `json_text`; Severin does not inspect that convention.

## JSON Frame Rules

The Rust boundary carries opaque serialized JSON frames. The only native validation is that each frame parses as JSON. The native layer does not require an object root, does not reserve fields, does not distinguish application success from application failure, and does not impose operation names.

Allowed examples are all equally opaque to Severin:

```json
null
```

```json
["anything", {"the_app": "owns_this_shape"}]
```

```json
{"userDefined": {"error": "also just data"}}
```

## Cancellation and Shutdown

The only engine-owned bridge failures are transport and lifetime failures:

- `App.close()` drops the transport and cancels pending reply targets.
- Page/document teardown invalidates pending reply targets for that document once the JS shim is wired in.
- A Python `write()` for an unknown or expired receipt fails as a bridge-disconnected/cancelled transport condition.

All normal application-level failures belong entirely to Python's chosen JSON protocol and should be delivered as ordinary JSON replies if Python wants JavaScript to see them.

## Thread and GIL Rules

Servo and WebView ownership in the Python extension is currently represented by an unsendable CPython object backed by raw native state, so the initial object is confined to the Python thread that created it. Future callbacks that touch Python must acquire the Python GIL before invoking Python code. They should not call Python while holding Servo locks, WebRender locks, network-loader locks, or mutable borrow state that Python re-entry could observe.

If a future bridge worker thread is introduced, it should pass serialized JSON frames and private receipts through Rust channels and acquire the GIL only at the Python boundary. Python remains the host authority; the JavaScript shim remains powerless queueing code.
