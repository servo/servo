# Python Embedding API

Severin has an experimental CPython native extension crate at `ports/severin-python/`. The extension module is named `severin` and embeds the host-side Servo instance directly in the Python process. It must not spawn a `severin` executable, bind a port, or communicate through HTTP, WebSocket, Unix socket, or localhost IPC.

## Download and install

The first distributable artifact is a GitHub Release wheel for Debian 12-compatible Linux x86_64 and one specific CPython minor version. It is not a universal Python wheel, not an ABI-independent wheel, and not a general cross-platform package.

Download the matching wheel from this repository's Releases page. The filename is explicit about the Python and platform tags, for example:

```text
severin-<version>-cp311-cp311-linux_x86_64.whl
```

Use the tag that appears in the generated Release asset for the interpreter in the Debian 12 build environment. Install it locally without PyPI or dependency downloads:

```bash
python3 -m pip install --user --no-deps ./severin-<version>-<python-tag>-linux_x86_64.whl
```

Verify the installed extension with normal CPython importing:

```bash
python3 -c 'import severin; print(severin.App)'
```

The wheel contains the in-process native extension. Installing or importing it does not download packages, contact PyPI, spawn a Severin helper executable, bind a port, or use HTTP, WebSocket, Unix socket, localhost IPC, or a daemon fallback.

## Initial API Shape

```python
import severin

app = severin.App(width=800, height=600, bridge=None)
app.load_path("app/index.html")

while True:
    app.pump()

    frame = app.read()
    if frame is not None:
        receipt, json_text = frame
        app.write(receipt, '{"ok": true}')
```

The Rust object behind `severin.App` owns:

- a `servo::Servo` instance built with `ServoBuilder`
- a `servo::WebView` attached to that Servo instance
- a software rendering context sized from the requested `width` and `height`
- a host-provided user script that installs the page-visible bridge on the top-level document
- the optional Python `bridge` object, retained only as host-side state and not used as a callback registry
- a native transport queue for opaque serialized JSON bridge frames

`App.load_path(path: str)` currently treats `path` as the package entry file. It sets the existing first-milestone package environment (`SERVORENA_PACKAGE_ID=com.example.app`, `SERVORENA_PACKAGE_ROOT=<entry parent>`) and navigates the embedded WebView to `asset://com.example.app/<entry filename>`. This is deliberately an adapter over the current package wall, not the final multi-package provider API.

`App.run()` remains a load-oriented helper: it spins Servo's in-process event loop and returns when the current load reaches `LoadStatus::Complete` or after `App.close()` has marked the instance closed. It does not replace the host pump loop needed for ongoing bridge activity after load.

`App.pump()` runs one bounded owner-thread turn: it spins Servo once, collects completed bridge evaluations, schedules one top-level-document outbound-drain evaluation when needed, and schedules or completes reply delivery. Python hosts should call it repeatedly while they want the embedded page and bridge to make progress. It does not start a background runtime, worker thread, helper process, daemon, socket, or callback thread.

`App.close()` drops the owned Servo/WebView state in-process so Servo's normal drop-time shutdown path can run. It also clears queued Python-visible frames and pending native reply targets.

## Page JavaScript API

For the loaded top-level document, Severin installs:

```js
const reply = await globalThis.severin.send(value);
```

`severin.send(value)` accepts JavaScript values that `JSON.stringify` can turn into strict JSON source. Values that cannot produce JSON source, such as cyclic structures or values that stringify to `undefined`, reject locally and do not allocate a native receipt.

The native layer treats the JSON source as opaque cargo. It validates that the source parses as JSON, allocates a private native receipt, queues `(receipt, json_text)` for Python, and never inserts the receipt into the application JSON payload.

Python replies with `App.write(receipt, json_text)`, where `json_text` must be valid JSON source. The original page Promise resolves to the parsed JavaScript value from that JSON source, not to a JSON string.

## Bridge Transport Model

The bridge is a mail slot with numbered native receipts, not an application protocol. The native layer does not define action names, capability names, permission rules, success/error conventions, request schemas, reply schemas, or a registry of host functions. It does not interpret application payload contents beyond the minimum needed to carry valid JSON.

The implemented flow is:

1. JavaScript calls `globalThis.severin.send(value)`.
2. The page shim serializes `value` with `JSON.stringify`. Local serialization failure rejects the Promise before native receipt allocation.
3. The page shim records the Promise in its own pending table using private per-document call bookkeeping.
4. `App.pump()` drains the page shim's outbound JSON frames through Servo's in-process JavaScript evaluation API.
5. Rust validates the JSON source, allocates a private native receipt, binds it to the current document identity and page call id, and queues `(receipt, json_text)` for Python.
6. Python calls `App.read()` and receives either `None` or `(receipt, json_text)`.
7. Python calls `App.write(receipt, json_text)` with arbitrary valid JSON for that receipt.
8. Subsequent `App.pump()` turns deliver the reply to the original pending page call only if the document identity still matches.
9. The JS shim parses the reply JSON source with `JSON.parse` and resolves the original Promise with that parsed value.

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

The minimum transport integrity implemented now is:

- receipts are single-use;
- `App.write()` validates reply JSON before consuming a receipt;
- invalid reply JSON fails without consuming the receipt;
- duplicate, unknown, expired, or already-consumed receipts fail as native transport/lifetime errors;
- receipts are bound to the originating top-level document identity;
- navigation or document teardown clears queued frames and native reply targets;
- a reply for a navigated-away document fails instead of reaching a later document in the same WebView;
- `App.close()` clears pending native reply targets.

There is no explicit discard/cancel API yet. If Python reads a request and chooses not to answer, the page Promise remains outstanding until Python replies or the document/App tears down.

## Current Limits

- The bridge is installed for the loaded top-level document only.
- Iframe support is not implemented.
- Only request/reply initiated by page JavaScript is implemented; there are no unsolicited host events or host-push messages.
- There is no Python callback API and no Python host-function registry.
- The optional `bridge` constructor argument does not define application operations or receive callbacks.
- There is no cancellation/discard API, timeout policy, quota, streaming, or backpressure policy yet.

## Thread and GIL Rules

Servo and WebView ownership in the Python extension is currently represented by an unsendable CPython object backed by raw native state, so the initial object is confined to the Python thread that created it. Future callbacks that touch Python must acquire the Python GIL before invoking Python code. They should not call Python while holding Servo locks, WebRender locks, network-loader locks, or mutable borrow state that Python re-entry could observe.
