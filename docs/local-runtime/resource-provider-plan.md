# Local Runtime Resource Provider Plan

This document sketches the first implementation path for turning Servo into a host-mediated local document/application runtime. The goal is not to delete networking first; it is to make all resource acquisition explicit and host-owned.

## Core Model

Servo should request resources through a contextual request object. The host resolves, authorizes, and returns bytes or a deterministic denial.

```rust
pub struct ResourceRequest {
    pub requested_url: Url,
    pub base_url: Option<Url>,
    pub initiator_url: Option<Url>,
    pub package_id: PackageId,
    pub origin: RuntimeOrigin,
    pub destination: ResourceDestination,
    pub mode: LoadMode,
    pub credentials: CredentialMode,
    pub cache_mode: CacheMode,
}

pub struct ResourceResponse {
    pub final_url: Url,
    pub mime_type: Mime,
    pub bytes: ResourceBody,
    pub cache_policy: CachePolicy,
    pub integrity: Option<IntegrityMetadata>,
    pub source_metadata: SourceMetadata,
}
```

The important inversion is: Servo may ask for resources, but only the host decides what is reachable.

## Initial Scheme Policy

Allowed in the first milestone:

- `asset://{package_id}/...` for package-relative resources rooted inside the active package.
- `bundle://runtime/...` for immutable runtime-owned resources.

Denied in the first milestone:

- `http://`
- `https://`
- `ws://`
- `wss://`
- `ftp://`
- `file://`
- `store://` as a fetchable resource
- `asset://` URLs for another package
- path traversal outside the package root

Deferred until real content requires them:

- `data:`
- `blob:`

## Request Flow

Every resource request should follow the same host-controlled path:

1. Receive a `ResourceRequest` from Servo.
2. Resolve `requested_url` against `base_url` when necessary.
3. Normalize and canonicalize the resolved URL.
4. Classify scheme and destination.
5. Check package, origin, and capability policy.
6. Load from the package or bundle backend.
7. Return `ResourceResponse` or a deterministic error.

Policy denials should be first-class outcomes, not lower-level network failures.

Initial instrumentation is deliberately late: `components/net/resource_thread.rs` logs built Fetch requests before they enter the legacy fetch/http/protocol path. The first v0 package implementation also lives at this seam. With `SERVORENA_PACKAGE_ID` and `SERVORENA_PACKAGE_ROOT` set, it authorizes `asset://{active_package_id}/...`, rejects raw and single-percent-decoded `..` traversal before mapping the path to a canonical file under the package root, returns bytes with simple extension-derived MIME, and denies package-mode remote HTTP(S), WebSocket, file, store, cross-package asset, missing file, I/O, and traversal/root-escape failures before legacy dispatch. This is intentionally not a package manager and not the final `ResourceProvider` boundary. In package mode, fallthrough beyond this wall is now treated as a bug/classification gap: known deny schemes return deterministic denials, and not-yet-routed schemes such as `bundle:`, `data:`, and `blob:` log `unsupported-unrouted` and complete with a deterministic network-error response before legacy fetch/protocol handling. Later work should move request construction earlier so original requested text, base URL, initiator URL, and destination-specific MIME errors are preserved more precisely.

A first earlier guard now exists in `ServoUrl` parsing/joining: raw requested text is inspected for path-boundary obfuscation before normalization. Ordinary percent encoding is still accepted, while encoded `.`, slash, backslash, NUL, and double-encoded forms of those bytes are logged and denied when package mode is enabled. This is a stopgap visibility/policy seam; the final provider request should carry both raw requested text and normalized final URL so denial reasons do not depend on URL parser error variants.

## Error Categories

The provider should distinguish at least these outcomes:

- `DeniedByPolicy` for disallowed schemes, capabilities, or cross-package access.
- `UnsupportedScheme` for schemes the runtime does not implement.
- `InvalidPath` for traversal or canonicalization failures.
- `NotFound` for missing package or bundle resources.
- `InvalidMime` for a resource that does not match the destination.
- `DecodeError` for bytes that cannot be consumed by the destination decoder.
- `IoError` for backend failures.

Good error categories are part of the developer experience. A missing local image should not look like a denied remote URL.

## First Milestone Package

The first acceptance package should be deliberately small:

```text
app/
  index.html
  styles.css
  main.js
  assets/logo.png
  fonts/app.woff2
```

The runtime should load:

- `asset://com.example.app/index.html`
- `./styles.css` from the document
- `./assets/logo.png` from HTML
- `./assets/logo.png` from CSS `url(...)`
- `./fonts/app.woff2` from `@font-face`
- `./main.js` as a classic script or module script

It should deterministically reject remote URLs, `file://`, traversal attempts, and `store://` fetches.

## CSS Subresource Context

CSS subresources must preserve two related but distinct URLs: the active document initiator and the stylesheet base that resolved the nested CSS reference. `@import` currently enters Servo through `components/script/stylesheet_loader.rs`, after Stylo resolves the imported URL against the parent stylesheet `UrlExtraData`; it is then fetched as `Destination::Style` and reaches the existing package wall. `@font-face src: url(...)` enters through `components/fonts/font_context.rs`; stylesheet-initiated font fetches now use the parsed stylesheet URL as the fetch referrer/base context while retaining the document URL as initiator context in local-runtime logging.

The final provider request shape should make this explicit instead of overloading referrer:

- `destination`: `Style`, `Font`, or `Image`
- `requested_url`: original CSS token text when available
- `base_url`: stylesheet URL for `@import`, `@font-face`, and CSS image `url(...)`
- `initiator_url`: active document URL
- `final_url`: resolved URL after stylesheet-relative resolution

CSS image `url(...)` remains an open mapping item.

## URL Resolution Provenance

Source-level provenance logging now exists before the final resource-thread policy wall for package-relevant document, external script, and module URL resolution. The important distinction is:

- `author_text` means the string is still visible at an owning document/module seam and is believed to be raw author input, such as a script `src` attribute or module specifier string.
- `resolver_input` means the shared URL parser/join layer sees a string, but the caller may already have decoded, rewritten, normalized, or otherwise transformed it.

Future `ResourceRequest` work should carry both the raw author spelling, when available, and the resolved `ServoUrl`. Without that request-shape change, final resource-policy logs can correlate by adjacent source-seam logs but cannot always prove which raw spelling produced a later normalized URL.
