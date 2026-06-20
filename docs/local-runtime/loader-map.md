# Servo Local Runtime Loader Map

This map is the working surface for routing Servo resource loading through a host-owned `ResourceProvider`. Each row should eventually point to concrete Servo code paths and migration notes.

| Category | Example | Current assumption to find | New behavior | Required context | First milestone |
| --- | --- | --- | --- | --- | --- |
| Initial document | `asset://com.example.app/index.html` | Navigation/document load asks for bytes | `ResourceProvider` request with `Document` destination | package, URL, origin | Yes |
| Stylesheet | `<link rel="stylesheet" href="./styles.css">` | Fetch stylesheet relative to document | `ResourceProvider` request with `Style` destination | document URL, base URL, MIME expectation | Yes |
| CSS image | `background: url("./assets/logo.png")` | Fetch CSS subresource relative to stylesheet | `ResourceProvider` request with `Image` destination | stylesheet URL as base, document origin | Yes |
| Font | `@font-face { src: url("./fonts/app.woff2") }` | Fetch font and notify layout after load | `ResourceProvider` request with `Font` destination | stylesheet URL as base, font MIME, origin | Yes |
| HTML image | `<img src="./assets/logo.png">` | Fetch image relative to document | `ResourceProvider` request with `Image` destination | document URL as base, image policy | Yes |
| Classic script | `<script src="./main.js"></script>` | Fetch and execute script | `ResourceProvider` request with `Script` destination | document URL as base, script policy | Yes |
| Module script | `<script type="module" src="./main.js"></script>` | Fetch module graph using URL identity | `ResourceProvider` request with `ModuleScript` destination | referrer URL, final URL, strict MIME | Yes/Phase 2 |
| Dynamic import | `import("./view.js")` | Fetch module from parent module base | `ResourceProvider` request with `ModuleScript` destination | parent module URL, origin | Phase 2 |
| Source map | `//# sourceMappingURL=main.js.map` | Fetch source map for diagnostics | Provider request with `SourceMap` destination | script URL, dev-mode policy | Later |
| JS fetch | `fetch("./data.json")` | Network API can fetch arbitrary URLs | Disabled or local-only provider request | JS origin, capability policy | Later |
| XHR | `new XMLHttpRequest()` | Legacy network API | Disabled or local-only provider request | JS origin, capability policy | Later |
| Worker | `new Worker("./worker.js")` | Fetch worker script | Disabled initially or provider request with `Worker` destination | parent origin, worker capability | Later |
| SVG external refs | SVG references external resource | May load nested resources | Provider-mediated or denied | SVG context, destination | Later |
| Media | `<video src="./clip.mp4">` | Fetch streamable media | Provider-mediated or disabled | media destination, streaming support | Later |
| Navigation | `<a href="./page.html">` | Navigate to URL | Host-approved navigation request | source document, user activation | Later |
| External URL | `<a href="https://example.com">` | Browser navigation/network | Deny or open externally via host capability | user activation, capability | Later |
| `file://` | `file:///tmp/foo` | Local filesystem access | Denied by policy | scheme policy | Yes: deny |
| `http(s)://` | `https://example.com/app.js` | Remote network access | Denied by policy | scheme policy | Yes: deny |
| `bundle://` | `bundle://runtime/default.css` | Runtime built-in resource | Provider loads immutable runtime asset | runtime version, destination | Yes |
| Store persistence | `app.store.get("theme")` | Not a fetch path | Host storage API, capability-gated | package, origin, store capability | Later |

## Discovery Checklist

For each row, identify:

1. The Servo crate/module where the request originates.
2. The URL/base URL context available at that point.
3. Whether the request is sync, async, or streaming.
4. The destination type and MIME expectations.
5. The cache identity and final URL behavior.
6. The failure propagation path.
7. The minimal change needed to log the request before enforcing policy.

The first implementation should add request logging before removing or disabling existing network code.

## Current Logging and Policy Seam

`components/net/resource_thread.rs` now logs every `CoreResourceManager::fetch` request after `RequestBuilder::build()` and before dispatch into the existing fetch/http/protocol machinery. This is a broad, late seam: it sees the concrete current URL, destination, referrer, origin, and credentials/cache mode, but it does not yet preserve the original unresolved attribute text or distinguish stylesheet base URLs from document base URLs for all caller categories. Treat this as request visibility and the first narrow policy enforcement point, not as the final `ResourceProvider` boundary.

At this seam, resolved `http://` and `https://` URLs are denied before entering the legacy HTTP loader. When `SERVORENA_PACKAGE_ID` and `SERVORENA_PACKAGE_ROOT` are both set, the same seam also handles first-milestone `asset://{package}/...` requests by checking the package host, checking the raw URL path for raw and single-percent-decoded `..` traversal before normalized segment use, canonicalizing the package root and target path, rejecting traversal/root escapes, reading bytes from the configured package root, assigning basic MIME types for `.html`, `.css`, `.js`, `.png`, and `.woff2`, and completing the request through Servo's existing response/chunk/eof target callbacks. Allowed package assets log `decision: package-asset`, `reason: PackageAssetAllowed`, MIME, and `local_path`. In package mode, `file://`, `store://`, `ws://`, `wss://`, wrong-package `asset://`, missing files, I/O failures, and path traversal are denied before legacy protocol handling. Outside package mode, existing `file://` local development behavior is preserved. Other schemes are now classified in package mode at this seam: known remote/file/store schemes return deterministic denial, while `ftp://`, `bundle://`, `data:`, `blob:`, and other not-yet-routed schemes log `decision: unsupported-unrouted` and complete with a deterministic network-error response instead of entering legacy fetch/protocol handling. Outside package mode, existing Servo behavior is preserved.

Raw URL laundering note: `components/url/lib.rs` now logs suspicious raw URL text in `ServoUrl::parse`, `ServoUrl::parse_with_base`, and `ServoUrl::join` before URL normalization. In package mode it denies encoded path-boundary obfuscation before `%2e%2e`-style input can become a clean normalized package path. The pinned fixture is `components/url/tests/local_runtime_raw_url.rs`, including the observed `./%2e%2e/secret.txt` join result. `Document::encoding_parse_a_url` still has a direct `url::Url::options().parse(...)` path with an encoding override, but now logs and package-mode-denies suspicious raw text immediately before that direct parse.

## CSS subresource notes

`components/script/stylesheet_loader.rs` is the current caller for linked stylesheets and CSS `@import`. The external `<link rel=stylesheet>` path creates a `Destination::Style` fetch from the element/document context. During stylesheet parsing, Stylo resolves `@import` against the parent stylesheet `UrlExtraData`; Servo's `ElementStylesheetLoader::request_stylesheet` receives that resolved URL and now logs the nested style request before handing it back to `load_with_element`, so package-mode policy enforcement still occurs at the net resource-thread wall.

`components/fonts/font_context.rs` handles `@font-face src: url(...)` after parsed stylesheet rules reach layout/font code. Stylo has already resolved the font URL from the stylesheet parser base. Stylesheet-initiated web-font requests now use the stylesheet URL, not just the document URL, as their fetch referrer/base context and log `destination: Font` before entering `fetch_async`. CSS image `url(...)` callers remain to be mapped separately.

## Module provenance logging notes

`components/script/script_module.rs` and `components/script/module_loading.rs` now add focused local-runtime module provenance logs under `script::script_module` / script module logging. The searchable prefixes are `[local-runtime module-resolution]`, `[local-runtime module-map]`, and `[local-runtime module-evaluation]`. These logs are restricted where practical to `asset://` and `bundle://` module URLs or edges whose importer/base is a local-runtime URL. They are intended to correlate module specifier resolution, module-map state, and top-level/dynamic module evaluation without changing package policy, resolution semantics, dynamic import behavior, or exception reporting.
