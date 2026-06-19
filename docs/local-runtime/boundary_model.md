# Local Runtime Boundary Model and Usual-Suspects Classification

## 1. Boundary model

This local runtime should behave like a **package-scoped document/app engine**, not like a web browser with carefully sandboxed internet features. The useful center is: package-authored HTML/CSS/JS may request package assets, but the host-owned resource gate decides whether those bytes exist and whether the request is allowed. The current main seam is `CoreResourceManager::fetch(...)`, which builds a request, calls `local_runtime_package_decision(...)`, serves allowed package assets, denies known blocked schemes, and otherwise falls through to legacy fetch machinery.

The practical distinction used by this plan:

- **Present in Servo source**: code exists in the repo, often because Servo is a browser engine.
- **Reachable in this build**: compiled/runtime paths may still call it.
- **Reachable from local-runtime package mode**: package mode with `SERVORENA_PACKAGE_ID` / `SERVORENA_PACKAGE_ROOT` can still get there.
- **Reachable from package-authored JS/HTML/CSS**: document code can cause the path, without embedder/private API involvement.

For this fork, “present in source” is acceptable for old web organs if local-runtime package content can **verified-cannot** reach them. The first obvious pressure point is that unknown schemes in package mode currently return `NotHandled`, after which they can fall through beyond the local-runtime package decision path.

## 2. Classification table for the 16 usual suspects

| # | Suspect | Recommended bucket | Why | Likely inspection target | Needs |
| ---: | --- | --- | --- | --- | --- |
| 1 | `CoreResourceManager::fetch` | **KEEP AND ROUTE THROUGH PACKAGE WALL** | This is the current central wall. It should remain the boring choke point for package asset loads and denials, not become a browser compatibility preservation layer. | `components/net/resource_thread.rs::CoreResourceManager::fetch` | **Both**: probes for all first-mile resource types, plus small code hardening around fallthroughs. |
| 2 | `local_runtime_package_decision` | **KEEP AND ROUTE THROUGH PACKAGE WALL** | This is the current package-mode policy classifier. It should become stricter, not broader: allow `asset://active-package/...`, deny known web/file/store schemes, classify unknowns as unsupported in package mode. | `components/net/resource_thread.rs::local_runtime_package_decision` | **Both**: probe unknown-scheme denial; likely tiny code change. |
| 3 | `local_runtime_path_has_traversal` + call site | **KEEP AND ROUTE THROUGH PACKAGE WALL** | Package-local paths are useful, including normal percent-encoded filenames, but traversal-shaped/layered encodings are not. Current concern is that this function sees `url.path()` after possible URL normalization. | `components/net/resource_thread.rs::local_runtime_path_has_traversal`; `RequestBuilder::build()` call site in `fetch` | **Both**: raw/encoded traversal fixtures and probably earlier raw-input logging/checking later. |
| 4 | `ServoUrl::parse_with_base`, `parse`, `join`, `path_segments` | **KEEP AND ROUTE THROUGH PACKAGE WALL** | URL resolution is necessary for relative package assets, but URL normalization can launder `%2e%2e` into package-local-looking paths before the gate sees them. | `components/url/lib.rs` URL wrapper methods; callers that use `base_url.join(...)` | **Probe fixture first**; code change only after exact laundering behavior is pinned down. |
| 5 | `Document::encoding_parse_a_url` | **KEEP AND ROUTE THROUGH PACKAGE WALL** | Document-relative URL parsing is needed for local documents, but should not hide suspicious raw input. | `components/script/dom/document/document.rs::encoding_parse_a_url` | **Probe fixture** for document/iframe/navigation-ish encoded traversal. |
| 6 | External script resolution | **KEEP AND ROUTE THROUGH PACKAGE WALL** | Classic and module scripts are core package app machinery, but only as package assets. Remote scripts, wrong-package scripts, file scripts, and traversal scripts should die at the gate. | `components/script/dom/html/htmlscriptelement.rs` external script URL resolution | **Probe fixture** for classic/module local allow and remote/traversal denial. |
| 7 | Module specifier resolution + module fetch | **KEEP AND ROUTE THROUGH PACKAGE WALL** | Static module scripts and dynamic `import()` are useful for local apps, but the module graph must stay package-scoped. | `components/script/script_module.rs::ModuleTree::resolve_url_like_module_specifier`; module `global.fetch(...)` call | **Both eventually**: probes for dynamic import, bare specifier failure, encoded traversal; code only if bypass found. |
| 8 | Stylesheet loader and CSS import/url loading | **KEEP AND ROUTE THROUGH PACKAGE WALL** | CSS, `@import`, images, and font URLs are normal package asset graph behavior. Preserve package-local relative resolution; deny everything outside. | `components/script/stylesheet_loader.rs`; CSS parser/resource loader paths | **Probe fixture first** for `@import`, CSS `url(...)`, stylesheet-relative base. |
| 9 | Web font downloader | **KEEP AND ROUTE THROUGH PACKAGE WALL** | Fonts are explicitly part of the package asset graph. They should work only as package files through `Destination::Font`. | `components/fonts/font_context.rs::RemoteWebFontDownloader::download` | **Probe fixture** for package font allow and remote/wrong-package denial. |
| 10 | Media loading | **DISABLE / GUT / BUILD OUT LATER** | Media is not a first-milestone package requirement and often involves streaming/backend code that may open URLs/files outside the main gate. Keep unsupported or package-only later; do not emulate browser media networking. | `components/script/dom/html/htmlmediaelement.rs::resource_fetch_algorithm` / `fetch_request`; platform media backends | **Manual inspection first**; likely code change later to deny or no-op URL media in package mode unless proven through gate. |
| 11 | WebSocket path | **DISABLE / GUT / BUILD OUT LATER** | WebSocket is a web-vehicle transport organ. There is no current package-local reason to preserve it. The separate path already needs special denial before scheme conversion. | `components/script/dom/websocket.rs`; `components/net/resource_thread.rs::websocket_connect` | **Both**: fixture for `new WebSocket(...)` denial; small code candidate to deny all WebSocket attempts in package mode regardless of scheme handling. |
| 12 | EventSource and sendBeacon | **DENY AT GATE** | These are network-shaped APIs. They can exist, but all external-resource attempts should hit ordinary local-runtime denial; no effort should preserve web semantics. | `components/script/dom/eventsource.rs`; `components/script/dom/navigator.rs::SendBeacon` | **Probe fixture** for denied remote/package-unsupported behavior. Code only if they bypass the gate. |
| 13 | XMLHttpRequest | **DENY AT GATE** for arbitrary external fetch; **KEEP AND ROUTE** only if later needed for package-local data | XHR is legacy arbitrary URL fetch. If local apps need data files, `fetch()` is enough; XHR can be allowed only insofar as it reaches the package wall. | `components/script/dom/xmlhttprequest.rs` | **Probe fixture** for remote/file/store/wrong-package denial and package-local data behavior if supported. |
| 14 | Workers / service workers / importScripts | **Workers: KEEP AND ROUTE or BUILD OUT LATER**; **Service workers: DISABLE / GUT / BUILD OUT LATER** | Dedicated workers may be useful for local apps if script loading is package-scoped. Service workers are browser empire machinery and should not mediate package policy. `importScripts` needs explicit package-wall proof. | `components/script/dom/workers/*`; `components/script/dom/workers/serviceworkerglobalscope.rs` | **Manual inspection + probes**. Likely service-worker disable candidate later; worker fixtures for constructor/importScripts remote denial. |
| 15 | HTTP/fetch/protocol/websocket backend modules | **PRESENT BUT SHOULD BE UNREACHABLE** | These may remain in Servo source, but package-authored content should not reach HTTP, WebSocket, protocol fallback, DNS/socket behavior, or custom protocol transport. | `components/net/http_loader.rs`; `components/net/fetch/methods.rs`; `components/net/protocols`; `components/net/websocket_loader.rs` | **Probe fixtures + trace/log proof**. Code only at entry gates, not broad backend surgery. |
| 16 | Android manifest permissions / intents | **DISABLE / GUT / BUILD OUT LATER** | Android `INTERNET`, external storage, and browser-like intent filters defeat “flat cannot network” and browser-intent cosplay is out of scope. | `support/android/apk/servoapp/src/main/AndroidManifest.xml` | **Code change later** for a local-runtime Android flavor/manifest; no runtime probe alone can prove OS-level cannot. |

## 3. Reachability separation by concept

### Present in Servo source

These are clearly present:

- HTTP/TLS/fetch/WebSocket/protocol backends remain in source and are imported/used by the resource thread.
- WebSocket has a distinct resource-channel path and backend handshake flow.
- Blob/file-token behavior remains after the local-runtime gate.
- Android browser permissions/intents are present in the app manifest according to the saved usual-suspects report.

### Reachable in this build

Likely yes for general Servo behavior:

- Normal `fetch(...)` falls through to legacy fetch for non-denied/non-package-handled schemes after the local-runtime checks.
- WebSocket outside package mode proceeds to scheme conversion and handshake fetch.
- `file://` outside package mode is intentionally preserved by current behavior, per the usual-suspects report.

### Reachable from local-runtime package mode

Current status from inspection:

- `asset://active-package/...` is reachable by design.
- `http` / `https` normal fetches appear denied before backend dispatch.
- `ws` / `wss` WebSocket attempts appear denied in `websocket_connect(...)` when package mode is active.
- `file` and `store` are denied only when `local_runtime_package_decision(...)` sees package mode.
- **Unknown schemes are the suspicious gap**: in package mode, `_ => NotHandled` means the request is logged and can continue into legacy fetch/protocol handling unless later layers reject it.

### Reachable from package-authored JS/HTML/CSS

This is the most important proof surface:

- Package HTML/CSS/JS likely can create fetch/XHR/script/module/CSS/font/image requests; those should all be package-wall probes.
- Package-authored JS can likely attempt WebSocket/EventSource/sendBeacon/worker/service-worker APIs; those should prove denial or unsupported behavior, not web compatibility.
- Package-authored markup can likely attempt iframe/embed/object/media/navigation paths; those should either be package-routed for local documents or disabled/denied.

## 4. Top 5 highest-value next manual inspections

1. **Unknown scheme fallthrough in package mode**
   - Inspect `local_runtime_package_decision(...)` and every path after `NotHandled` in `CoreResourceManager::fetch`.
   - Why it matters: unknown schemes are the cleanest “not explicitly denied” gap in the current gate.
   - Remedy: in package mode, return a deterministic denial for all schemes except explicitly allowed package schemes and intentionally deferred local schemes.

2. **Raw URL laundering before the gate**
   - Inspect `ServoUrl::join/parse_with_base`, `Document::encoding_parse_a_url`, script URL resolution, and module specifier resolution.
   - Why it matters: `%2e%2e` can be normalized before `local_runtime_path_has_traversal(...)` sees the path.
   - Remedy: preserve/log raw requested text earlier, but only after fixtures pin down exact behavior.

3. **CSS `@import` and nested CSS `url(...)` base handling**
   - Inspect stylesheet loader and CSS resource code.
   - Why it matters: CSS is first-milestone package graph functionality, and stylesheet-relative base URLs must work while remote/traversal fails.
   - Remedy: route all CSS subresources through the same package wall with destination and stylesheet base context.

4. **Worker/service-worker/importScripts split**
   - Inspect dedicated/shared worker constructors, `importScripts`, module workers, and service-worker registration/script loading.
   - Why it matters: dedicated workers may be useful local app machinery, but service workers are policy-mediating browser empire machinery.
   - Remedy: keep workers only if script graph is package-gated; disable service workers in package mode unless a concrete local use case appears.

5. **Media/player backend URL handling**
   - Inspect `HTMLMediaElement::resource_fetch_algorithm`, `fetch_request`, and media backend handoff.
   - Why it matters: media backends sometimes open URLs/files independently.
   - Remedy: deny media URL loads in package mode until proven routed through the package wall; later allow package-local media only if needed.

## 5. Top 5 highest-value probe fixtures

1. **Unknown scheme denial fixture**
   - Package page tries `fetch("gopher://example.test/x")`, `<img src="custom://x">`, `<script src="weird://x">`, and CSS `url(weird://x)`.
   - Expected: deterministic local-runtime denial, no backend/protocol fallthrough.

2. **Encoded traversal laundering matrix**
   - Package page references `asset://com.example.app/%2e%2e/secret.js`, `.%2e/secret.js`, `%252e%252e/secret.js`, encoded slash/backslash variants, and normal filenames with harmless percent encoding.
   - Expected: traversal/layered suspicious forms denied; normal encoded filenames allowed.

3. **Package asset graph fixture**
   - `index.html` loads stylesheet, CSS `@import`, CSS image, HTML image, font, classic script, module script, dynamic import, and package-local JSON/data via `fetch()`.
   - Expected: all package-local assets route through the gate and work; no remote fallback.

4. **Web-vehicle API denial fixture**
   - Package JS attempts WebSocket, EventSource, sendBeacon, service-worker registration, worker `importScripts("https://...")`, and XHR/fetch to remote/file/store/wrong-package URLs.
   - Expected: denied or unsupported, with no network/socket/file access.

5. **Frame/embed/object/media fixture**
   - Package HTML attempts `<iframe src="https://...">`, `<iframe src="asset://wrong-package/...">`, `<embed>`, `<object>`, `<video src="https://...">`, `<audio src="file://...">`.
   - Expected: deny/unsupported unless package-local iframe/document is intentionally allowed through the same wall.

## 6. One-file obvious next patch candidates

No patch was applied as part of this plan. The most obvious one-file candidate is:

### Candidate: deny unknown schemes in package mode inside `local_runtime_package_decision`

**File:** `components/net/resource_thread.rs`

**Tiny change shape:** inside `local_runtime_package_decision(...)`, change the package-mode `_ => NotHandled` branch to a deterministic denial such as `UnsupportedSchemeDeniedByLocalRuntime`, while preserving explicit `asset` allow-routing and explicit denied schemes.

**Why this fits the boundary model:** package mode should not be “anything Servo might know how to fetch.” It should be “only package wall or deterministic denial.” Unknown scheme fallthrough is web-browser-shaped behavior.

**Why this is safe conceptually:** it does not delete backend code, does not refactor fetch, and does not decide the future of `data:`/`blob:` in detail. It simply prevents package-mode authored requests with unclassified schemes from bypassing the local-runtime classifier.

**Needed before/with it:** one small probe fixture for an unknown scheme, because this is exactly the kind of “verified cannot” behavior the project wants.
