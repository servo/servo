# Usual Suspects Report: Servo Local-Runtime Network / External-Resource Escape Paths

## 1. Central gate

**Main gate:** `components/net/resource_thread.rs`, centered on:

- `CoreResourceManager::fetch(...)` — the primary late fetch seam after `RequestBuilder::build()` and before legacy fetch/http/protocol dispatch. It calls `local_runtime_package_decision(...)`, logs via `log_local_runtime_resource_request(...)`, serves allowed package assets, returns denial responses, and otherwise falls through to Servo’s normal fetch path.
- `local_runtime_package_decision(...)` — the package-mode policy classifier for `asset`, remote, WebSocket, `file`, and `store` schemes.
- `local_runtime_path_has_traversal(...)` — raw/single-percent-decoded traversal screening currently applied to `url.path()`.
- `CoreResourceManager::websocket_connect(...)` — a separate WebSocket entry path that denies `ws`/`wss` in package mode before converting WebSocket schemes to HTTP(S) for handshake fetch.

**Schemes currently classified at the logging/policy seam:**

- `asset` / `bundle`: logged as “unrouted” by the generic logging classifier; `asset` is actually handled by `local_runtime_package_decision(...)` when package mode is active.
- `http` / `https`: denied by `local_runtime_package_decision(...)` in package mode, and also denied unconditionally after logging in `CoreResourceManager::fetch(...)`.
- `ws` / `wss`: denied by `local_runtime_package_decision(...)` only when they arrive through normal fetch, and separately denied in `websocket_connect(...)` in package mode before handshake conversion.
- `file`: denied only in package mode by `local_runtime_package_decision(...)`; otherwise the generic classifier still labels it legacy-path.
- `store`: denied only in package mode by `local_runtime_package_decision(...)`; otherwise the generic classifier still labels it legacy-path.
- Other schemes: generally “NotHandled” at this gate and then fall through to blob/file/protocol/fetch machinery unless another path rejects them.

**Request destinations likely passing through `CoreResourceManager::fetch(...)`:**

The central gate sees `RequestBuilder.destination`, including at least `Document`, `Style`, `Script`, module-related destinations, `Image`, `Font`, `ServiceWorker`, `Worker`, XHR/fetch/EventSource/beacon destinations where they use `global.fetch(...)` or `CoreResourceMsg::Fetch`. The gate’s timing classification explicitly treats `Destination::Document` as navigation timing and all other destinations as resource timing.

## 2. Raw URL / normalization concern

**Primary concern:** many callers resolve strings into `ServoUrl` before the resource gate sees them. The gate receives `request.current_url()` after `RequestBuilder::build()`, so original attribute/source text may already be lost.

Exact files/functions worth inspecting:

1. **`components/url/lib.rs::ServoUrl::parse_with_base` / `ServoUrl::parse` / `ServoUrl::join` / `ServoUrl::path_segments`**
   - These wrap the `url` crate parser/joiner and are the likely normalization point for `%2e%2e`, dot segments, and serialized paths before local-runtime checks see `url.path()` or `url.path_segments()`.
2. **`components/script/dom/document/document.rs::Document::encoding_parse_a_url`**
   - This applies URL parser options with base URL and encoding override, which means HTML/document-facing URLs may be normalized before network policy sees them.
3. **`components/script/dom/html/htmlscriptelement.rs` script URL resolution**
   - Classic/module script `src` values are resolved with `base_url.join(&src)` before script fetching, so original `src` text is not preserved at the resource-thread gate.
4. **`components/script/script_module.rs::ModuleTree::resolve_url_like_module_specifier`**
   - Dynamic and static module specifiers use `ServoUrl::parse_with_base(...)` or `ServoUrl::parse(...)`, so module graph traversal-like encodings may normalize before fetch.
5. **`components/net/resource_thread.rs::local_runtime_path_has_traversal`**
   - This checks `url.path()` and single-percent-decoded path text, but if `%2e%2e` was already removed/normalized by URL parsing, this function cannot distinguish “normalized away” from a genuinely package-local path.

## 3. API usual suspects

| API / resource type | Likely gate status | Notes |
| --- | --- | --- |
| `fetch()` | **Likely same gate** | Script fetch sends `CoreResourceMsg::Fetch` through the core resource thread. |
| `XMLHttpRequest` | **Likely same gate** | XHR builds a request and calls `global.fetch(...)`, which should route to the same core fetch machinery. |
| dynamic `import()` | **Likely same gate, normalization concern** | Module specifiers resolve through `ModuleTree::resolve_url_like_module_specifier(...)`, then module loading calls `global.fetch(...)`. |
| module scripts | **Likely same gate, normalization concern** | `<script type=module src>` is resolved via `base_url.join(&src)` and then fetched as module script machinery. |
| classic scripts | **Likely same gate, normalization concern** | Classic script `src` resolves via `base_url.join(&src)` and calls `fetch_a_classic_script(...)`. |
| CSS `@import` | **Unclear from this quick pass** | Stylesheet loads go through `document.fetch(...)` with `Destination::Style`; CSS parser/import subloads need direct confirmation in style/CSS loader code. |
| fonts | **Likely same gate** | Web fonts build `RequestBuilder` with `Destination::Font` and use the core resource thread. |
| images | **Likely same gate** | Layout image loading builds `Destination::Image` request and calls `document.fetch_background(...)`. |
| audio/video/media | **Unclear / likely separate media backend after URL selection** | HTML media resolves URLs, stores `resource_url`, then calls `fetch_request(...)`; inspect whether that fetch goes through net gate or media backend can open URLs itself. |
| workers | **Likely same gate for script fetch, but inspect worker constructors** | Service worker script loading uses `load_whole_resource(...)`, which sends `CoreResourceMsg::Fetch`; dedicated/shared worker script paths need the same confirmation. |
| service workers | **Likely same gate for script fetch; separate behavioral risk** | Service worker script fetch goes through `load_whole_resource(...)`; service-worker interception and registration logic remain usual suspects because they can mediate fetches internally. |
| WebSocket | **Definitely separate path plus fetch backend** | DOM WebSocket sends `CoreResourceMsg::Fetch` with `FetchChannels::WebSocket`; resource thread dispatches to `websocket_connect(...)`, which has a package-mode denial before converting `ws`/`wss` to `http`/`https`. |
| EventSource | **Likely same gate** | EventSource calls `global.fetch(...)` for initial and retry fetches. |
| sendBeacon | **Likely same gate** | `Navigator::SendBeacon` calls `global.fetch(...)` with keepalive. |
| navigation / iframe / embed / object | **Mixed / needs focused inspection** | Document/navigation fetches likely hit `Destination::Document`; iframe URL resolution is earlier in script code; object/embed/plugin/media-object paths may be separate or disabled depending on resource type. The current gate treats `Destination::Document` specially for timing but not separately for policy. |

## 4. Network backend usual suspects

### Present in source

These modules still look capable of network or external-resource work:

1. **HTTP/TLS client construction**
   - `components/net/resource_thread.rs` imports and constructs HTTP state with `create_http_client(...)`, TLS config, HSTS, cookies, auth cache, and HTTP cache.
2. **Fetch implementation**
   - `components/net/fetch/methods.rs` contains the main fetch state machine and calls HTTP/scheme fetch paths. The resource thread falls through to `fetch(request, &mut sender, &context).await` for non-denied/non-package-handled schemes.
3. **HTTP loader**
   - `components/net/http_loader.rs` remains the HTTP network/cache/redirect/CORS implementation; resource thread imports `http_redirect_fetch` and has an unconditional remote-scheme denial before normal fetch only for `http`/`https` at the central seam.
4. **WebSocket loader**
   - `components/net/websocket_loader.rs` remains present and is used by `websocket_connect(...)` after scheme conversion and handshake request creation.
5. **File manager / blob file-token path**
   - `components/net/resource_thread.rs` still has blob/file token handling after the local-runtime gate, including fallback token acquisition for blob URLs that were not claimed before net.
6. **Protocol registry**
   - `components/net/protocols` remains reachable after fallthrough through `FetchContext.protocols`; this matters for non-HTTP schemes and custom protocol handling.

### Reachable from local-runtime mode

- **HTTP(S):** appears denied at `CoreResourceManager::fetch(...)` before normal fetch dispatch for all normal fetch-channel paths, even outside package mode.
- **WebSocket:** appears denied in package mode in `websocket_connect(...)` before `ws`/`wss` are converted to `http`/`https`; outside package mode it remains reachable.
- **`file://`:** denied only when package mode is active; outside package mode it remains intentionally preserved.
- **`store://`:** denied only when package mode is active; outside package mode classification still says legacy path.
- **Blob/data/custom protocols:** not proven denied by this pass; blob explicitly has post-gate handling, and other schemes fall through unless the fetch layer rejects them.

## 5. Android-specific usual suspects

**Relevant finding:** the Android application manifest grants network and storage authority and registers browser-like external intents.

- `android.permission.INTERNET` is present in the app manifest.
- `READ_EXTERNAL_STORAGE` and `WRITE_EXTERNAL_STORAGE` are also present, which matters for a “flat cannot access outside package root” posture.
- MainActivity is exported and accepts browser intents for `http`, `https`, `data`, `javascript`, and later `file`/`http`/`https` MIME-associated intents.

**Interpretation:** Android currently does **not** provide “verified cannot network” at the platform manifest layer. Even if local-runtime policy denies remote fetches, the packaged Android app still has OS-level internet permission and browser-shaped intent ingress.

## 6. Recommended manual inspection list

1. **`components/net/resource_thread.rs::CoreResourceManager::fetch`**
   - Why: this is the main policy seam and must be audited for every branch after `local_runtime_package_decision(...)`.
   - Remedy: ensure package mode denies or explicitly classifies every scheme before `spawn_task(fetch(...))`.
2. **`components/net/resource_thread.rs::local_runtime_package_decision`**
   - Why: this is the current scheme/package/path decision point.
   - Remedy: make all local-runtime-denied schemes first-class here and decide whether unknown schemes should be `UnsupportedScheme` rather than `NotHandled` in package mode.
3. **`components/net/resource_thread.rs::local_runtime_path_has_traversal` plus call site**
   - Why: it only sees `url.path()` after prior URL parsing/joining may have normalized `%2e%2e`.
   - Remedy: carry raw requested string/base into the gate or add earlier logging/checks at URL construction sites.
4. **`components/url/lib.rs::ServoUrl::parse_with_base`, `parse`, `join`, `path_segments`**
   - Why: these are central wrappers around URL parser normalization.
   - Remedy: document exactly how encoded dot segments and encoded slash/backslash are serialized before `resource_thread` sees them.
5. **`components/script/dom/document/document.rs::encoding_parse_a_url`**
   - Why: document-originated HTML URLs may be normalized here with document encoding context.
   - Remedy: log raw input plus resulting `ServoUrl` for local-runtime audit builds.
6. **`components/script/dom/html/htmlscriptelement.rs` external script resolution**
   - Why: classic and module script `src` values are resolved before the network gate.
   - Remedy: preserve raw `src`, base URL, and resolved URL in request metadata/logging.
7. **`components/script/script_module.rs::ModuleTree::resolve_url_like_module_specifier` and module fetch call**
   - Why: static and dynamic module imports are common traversal/remote-loading paths.
   - Remedy: ensure module specifier raw text is logged and policy is applied before normalized graph fetch identity hides suspicious input.
8. **`components/script/stylesheet_loader.rs` and CSS import/url loading paths**
   - Why: stylesheet loads are confirmed, but CSS `@import` and nested `url(...)` bases need exact path confirmation.
   - Remedy: map each CSS subresource destination to a request with stylesheet URL as base and local-runtime log context.
9. **`components/fonts/font_context.rs::RemoteWebFontDownloader::download`**
   - Why: fonts are an explicit first-milestone package asset type and use `Destination::Font`.
   - Remedy: verify font URLs always pass through core resource fetch and cannot be opened by platform font backends.
10. **`components/script/dom/html/htmlmediaelement.rs::resource_fetch_algorithm` / `fetch_request`**
    - Why: media may cross from DOM URL selection into media backends.
    - Remedy: verify `fetch_request` uses core resource fetch for URL resources and that GStreamer/platform media code cannot open remote/file URLs independently.
11. **`components/script/dom/websocket.rs` and `components/net/resource_thread.rs::websocket_connect`**
    - Why: WebSocket has a separate channel and transforms `ws`/`wss` into `http`/`https` for handshake.
    - Remedy: keep the package-mode denial before scheme conversion and consider making the denial unconditional for local-runtime builds.
12. **`components/script/dom/eventsource.rs` and `components/script/dom/navigator.rs::SendBeacon`**
    - Why: these APIs can create long-lived or keepalive network fetches.
    - Remedy: confirm all retries/keepalive records still hit `CoreResourceManager::fetch` and inherit local-runtime denial.
13. **`components/script/dom/xmlhttprequest.rs`**
    - Why: XHR is a legacy arbitrary URL fetch surface.
    - Remedy: verify all async/sync XHR modes use `global.fetch(...)` or `load_whole_resource(...)` and cannot bypass the resource thread.
14. **`components/script/dom/workers/*` and `components/script/dom/workers/serviceworkerglobalscope.rs`**
    - Why: workers create new globals with their own base URLs and can fetch from inside worker contexts.
    - Remedy: confirm worker script load, importScripts/module workers, and worker-global fetch all use the same gate and package identity.
15. **`components/net/http_loader.rs`, `components/net/fetch/methods.rs`, `components/net/protocols`, `components/net/websocket_loader.rs`**
    - Why: these remain fully capable backend implementations.
    - Remedy: treat them as present-but-should-be-unreachable in package mode, then prove by tracing all entry points rather than deleting them.
16. **`support/android/apk/servoapp/src/main/AndroidManifest.xml`**
    - Why: Android grants `INTERNET`, storage permissions, and browser-like external intent ingress.
    - Remedy: for local-runtime Android builds, remove `INTERNET`, remove broad storage permissions, and remove `http`/`https`/`file` browser intent filters.

## 7. Milestone 2 execution shape — module graph first, browserland contained

Milestone 2 should not execute this report's complete target matrix strictly top-to-bottom. The matrix is useful as coverage, but the next engineering slice should be opinionated: preserve and prove the package machinery that makes local apps pleasant to write, while explicitly gutting or deferring browser machinery that would drag the fork back toward general web compatibility.

The first-milestone baseline is already strong: package document, HTML image, stylesheet, CSS `@import`, nested font, nested CSS image, and `main.js` execution work; `http`, `https`, `ftp`, `ws`, `wss`, `file`, `store`, cross-package `asset`, and traversal-shaped package escapes are denied. Milestone 2 should now convert the most important remaining assumptions into evidence.

### Immediate slice A: External modules, static imports, and dynamic `import()`

Inspect `components/script/dom/html/htmlscriptelement.rs` external module script resolution, `components/script/script_module.rs::ModuleTree::resolve_url_like_module_specifier`, and the module fetch path that calls `global.fetch(...)`.

Why it matters: modules are the next real app-making capability after `main.js`. A local runtime that can execute one package script but has a porous, inconsistently based, or broken module graph is not yet a comfortable programming substrate.

Remedy: treat module entry scripts, static imports, and dynamic `import()` as one probe unit. Prove package-local module entry, relative static import, nested relative static import, and dynamic import all resolve against the expected module/document base and hit the package wall. In the same fixture family, prove remote specifiers, wrong-package `asset://` specifiers, `file://`, bare specifiers, literal traversal, encoded dot traversal, encoded slash/backslash traversal, and double-encoded traversal fail deterministically with useful logs.

Current Slice A Scoreboard:
* same-package dynamic import:      PASS
* top-level await success:          PASS
* top-level await expected reject:  PASS
* cross-package asset import:     DENIED: PackageIdMismatch
* remote module import:             DENIED: RemoteSchemeDeniedByLocalRuntime
* bare specifier:                  rejected before fetch
* encoded traversal:                rejected before normalization
* literal ../:                      normalized within package, then NotFound

### Immediate slice B: Raw author-text, base URL, and resolved-URL logging at module/document seams

Inspect `ServoUrl::join`, `ServoUrl::parse_with_base`, `ServoUrl::parse`, `Document::encoding_parse_a_url`, `components/script/dom/html/htmlscriptelement.rs`, and `components/script/script_module.rs::ModuleTree::resolve_url_like_module_specifier`.

Why it matters: `%2e%2e`, encoded slashes, and odd bases can be normalized before `components/net/resource_thread.rs::local_runtime_path_has_traversal(...)` sees the request. Without raw author-text/base/resolved-URL evidence at the module and document seams, “the late traversal check is enough” remains an assumption.

Remedy: do not start with a giant early rewrite. First add or tighten logging/probes that show the raw author text, the base URL used for resolution, the resolved `ServoUrl`, the destination/module context, and the final package-wall decision. Use those fixtures to decide where raw requested text must be carried forward into the eventual host-owned `ResourceProvider` request shape.

### Immediate slice C: WebSocket as the separate early denial proof

Inspect `components/script/dom/websocket.rs` and `components/net/resource_thread.rs::websocket_connect`.

Why it matters: WebSocket has a distinct resource-thread path and converts `ws`/`wss` into `http`/`https` for the handshake. It is the obvious case where “normal fetch reaches the wall” may not prove the whole wall.

Remedy: add focused `new WebSocket(...)` probes for `ws://`, `wss://`, package-looking inputs, wrong-package inputs, and odd encoded inputs. Keep denial before scheme conversion and make package-mode WebSocket failure explicit, deterministic, and separately logged from ordinary fetch denials.

## 8. Milestone 2 preserve/prove bucket

These are the package-runtime capabilities worth preserving because they make sealed local documents feel like usable applications rather than single-file demos. They should be routed through the package wall and proved with focused fixtures before broadening scope.

### Modules and dynamic imports

Inspect `components/script/dom/html/htmlscriptelement.rs` and `components/script/script_module.rs` as the first preserve/prove priority.

Why it matters: modules define the practical programming model for non-trivial local package apps.

Remedy: execute the Immediate slice A and B work before spending comparable effort on legacy network-shaped APIs.

### CSS graph

Inspect `components/script/stylesheet_loader.rs` and the style/layout image-request path for linked stylesheets, CSS `@import`, CSS `url(...)` images, and stylesheet-relative bases.

Why it matters: the first milestone proves CSS works, but the runtime still needs exact evidence that every nested CSS edge carries the right stylesheet base and destination into the package wall.

Remedy: keep CSS graph work as preserve/prove work: add fixtures for nested `@import`, CSS image `url(...)`, and stylesheet-relative paths, with logs that show document initiator, stylesheet base, resolved URL, destination, and final decision.

### Fonts

Inspect `components/fonts/font_context.rs::RemoteWebFontDownloader::download` and platform font handoff points.

Why it matters: package-local fonts are part of authoring a polished offline document/app, but platform font consumers must not independently open URLs or files outside the provider boundary.

Remedy: prove package font allow and remote/wrong-package/traversal denial, then document that only bytes returned through the core resource fetch are handed to font consumers.

### Dedicated workers, later and only if useful

Inspect `components/script/dom/workers/*` for dedicated worker constructors, worker-global fetch, module workers, and `importScripts`.

Why it matters: dedicated workers may buy something for local apps, but they also create new globals, new bases, and another place to accidentally reintroduce ambient fetch authority.

Remedy: defer until modules/CSS/fonts are boring. If preserved, prove worker script loading, worker-global fetch, and `importScripts` use the same package identity and package wall; otherwise make package-mode unsupported behavior explicit.

## 9. Milestone 2 gut/defer bucket

These APIs are browser machinery, not necessary package machinery. The goal is not to sanctify all of browserland; it is to keep the local-runtime authority story small and inspectable.

### WebSocket

Inspect `components/script/dom/websocket.rs` and `components/net/resource_thread.rs::websocket_connect` early despite being in the gut/defer bucket.

Why it matters: it is both browser transport machinery and a separate path, so it deserves early proof that it cannot escape package-mode policy.

Remedy: execute Immediate slice C, then keep WebSocket disabled or deterministically denied in package mode unless a future host-mediated, non-network-shaped local capability is explicitly designed.

### EventSource and sendBeacon

Inspect `components/script/dom/eventsource.rs` and `components/script/dom/navigator.rs::SendBeacon`.

Why it matters: both are network-shaped APIs with retry or keepalive semantics that do not help the sealed package runtime enough to justify preserving web behavior.

Remedy: defer broad compatibility work. Add denial probes only to prove package-mode requests hit ordinary local-runtime denial and that retries/keepalive behavior cannot bypass `CoreResourceManager::fetch`.

### XMLHttpRequest

Inspect `components/script/dom/xmlhttprequest.rs` only after the module/fetch package story is stable.

Why it matters: XHR is legacy arbitrary URL fetch. If local apps need package data, modern `fetch()` or an explicit host data API is enough; XHR should not drive the architecture.

Remedy: defer preservation. Later, either deny XHR in package mode or allow only the subset that demonstrably reaches the same package wall with package-local URLs and deterministic denials.

### Service workers

Inspect `components/script/dom/workers/serviceworkerglobalscope.rs` and service-worker registration paths only to identify disable points.

Why it matters: service workers are browser empire machinery and can become a second policy plane, which is the opposite of a host-owned package boundary.

Remedy: gut or defer service workers for package mode. Do not let them mediate local-runtime fetch policy unless a future design deliberately assigns them a narrow, host-controlled role.

### URL media

Inspect `components/script/dom/html/htmlmediaelement.rs::resource_fetch_algorithm`, `fetch_request`, and platform media backend URL handoff points.

Why it matters: media often involves streaming and platform backends that may open URLs/files outside the normal gate, and media is not necessary for the first comfortable programming substrate.

Remedy: likely deny/no-op URL media in package mode until there is a provider-mediated local-media story. If local media is later needed, prove bytes or streams come only from package-authorized provider responses.

## 10. Milestone 2 cross-cutting gate and platform checks

These checks support both buckets, but they should serve the chosen slices rather than becoming an exhaustive browser audit by themselves.

### Central package wall and scheme classification

Inspect `components/net/resource_thread.rs::CoreResourceManager::fetch`, `local_runtime_package_decision(...)`, and every package-mode fallthrough after the decision.

Why it matters: the central wall remains the current choke point for package assets and denials, so unknown schemes or unclassified fallthroughs can quietly preserve browser behavior.

Remedy: as module/WebSocket/CSS/font probes are added, harden any discovered fallthrough so package mode serves package assets, returns deterministic denial, or logs an explicit unsupported/unrouted decision. Classify unrecognized fetchable schemes as `UnsupportedScheme` in package mode unless documented otherwise.

### Backend unreachability

Inspect `components/net/http_loader.rs`, `components/net/fetch/methods.rs`, `components/net/protocols`, and `components/net/websocket_loader.rs` from package-authored entry points.

Why it matters: these backend modules may remain present, but package content must not reach HTTP, DNS/socket, protocol fallback, custom protocol, or WebSocket transport behavior.

Remedy: collect trace/log proof from the chosen probes that denied package-mode requests stop before backend dispatch. Prefer entry-gate hardening over deleting backend machinery that may still serve non-package Servo use cases.

### Android authority posture

Inspect `support/android/apk/servoapp/src/main/AndroidManifest.xml` for `INTERNET`, storage permissions, exported activities, and browser-like intent filters.

Why it matters: runtime denial logs cannot prove a local-runtime Android build has no OS-level network or broad filesystem authority if the manifest still grants those permissions.

Remedy: design a local-runtime Android flavor or manifest variant that removes `INTERNET`, removes broad external storage access, and drops `http`/`https`/`file` browser intent ingress. Treat this as a code-change-later item because probes alone cannot establish OS-level absence of authority.
