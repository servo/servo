
## Project Shape

This fork is exploring Servo as the base for a local, offline-first document/application runtime.

The goal is not to build a product, chase browser nightlies, or keep pace with the modern web indefinitely. The goal is to capture enough mature web-engine capability to make local tools, documents, editors, games, and small applications feel powerful without giving them ambient network authority.

Think of this as a durable personal runtime appliance:

* local HTML/CSS/JS-style application support
* real layout/style/rendering machinery where Servo already provides it
* package-scoped resources
* host-mediated loading
* deterministic denials for forbidden outside access
* no assumption of an audience, app-store release, or public platform roadmap

Favor boring, inspectable, repeatable changes over cleverness.

## North Star

Do not start by “deleting networking.”

Start by making every resource acquisition explicit, contextual, logged, and host-owned.

The core architecture is:

```text
Servo may ask for resources.
The host decides what exists.
The host decides what is reachable.
The host returns bytes or a deterministic denial.
```

This means the center of the project is a `ResourceProvider`-style boundary, even if Servo does not currently expose one perfect seam.

The intended division is:

```text
Servo owns:
- HTML parsing
- CSS/style
- DOM/script machinery
- layout
- rendering/compositing
- document behavior

The local runtime host owns:
- package identity
- URL resolution
- resource authorization
- local asset loading
- persistence
- capability grants
- external effects
```

If a feature crosses from document machinery into external authority, it belongs behind the host boundary.

## Required Local Runtime Docs

Keep these documents current as the work progresses:

* `/docs/local-runtime/loader-map.md`
* `/docs/local-runtime/resource-provider-plan.md`

Do not duplicate their full contents here. Treat them as the working design surface.

When discovering a new resource path, request origin, Servo crate, failure mode, cache identity issue, or hidden network-adjacent behavior, update the relevant doc.

If a change affects the design model, document it before or alongside the code change.

## Progress Documentation

Maintain a progress log for this work. If no file exists yet, create one at:

```text
/docs/local-runtime/progress.md
```

Use it to record:

* date / commit range
* what was inspected
* which Servo crates/modules were touched
* what resource paths were discovered
* what was logged successfully
* what was denied successfully
* what still reaches old network/resource assumptions
* failed approaches and why they failed
* open questions for the next pass

This project should leave a trail. Future work should not require rediscovering the same Servo seams from scratch.

## First Engineering Rule: Log Before Cutting

Before removing, disabling, or replacing existing behavior, add logging wherever possible.

The first useful win is a clear request log, not a dramatic deletion.

A good log entry should answer:

```text
destination:
requested URL:
base URL:
initiator URL:
origin/package:
Servo crate/module:
decision:
final URL:
mime:
error/denial reason:
```

Example shape:

```text
[local-runtime resource-request]
  destination: Style
  requested: ./styles.css
  base: asset://com.example.app/index.html
  initiator: asset://com.example.app/index.html
  package: com.example.app
  decision: allow
  final: asset://com.example.app/styles.css
  mime: text/css
```

And for denial:

```text
[local-runtime resource-request]
  destination: Image
  requested: https://example.com/logo.png
  base: asset://com.example.app/index.html
  initiator: asset://com.example.app/index.html
  package: com.example.app
  decision: deny
  reason: RemoteSchemeDenied
```

If a resource path cannot yet be routed through the new provider, log that fact explicitly.

## First Milestone

The first milestone is deliberately narrow.

Load a small package:

```text
app/
  index.html
  styles.css
  main.js
  assets/logo.png
  fonts/app.woff2
```

Required behavior:

* load `asset://com.example.app/index.html`
* resolve `./styles.css` relative to the document
* resolve HTML images relative to the document
* resolve CSS `url(...)` relative to the stylesheet
* resolve `@font-face` relative to the stylesheet
* load and execute `./main.js`
* deny `http://`
* deny `https://`
* deny `ws://`
* deny `wss://`
* deny `ftp://`
* deny `file://`
* deny `store://` as a fetchable resource
* deny cross-package `asset://` access
* deny path traversal outside the package root

This milestone is more meaningful than a screenshot. The success condition is: local package resources work, forbidden outside resources fail predictably, and the log explains why.

## FIRST MILESTONE ACHIEVED 🎉🥳🎉🥳🎉🥳🎉🥳

_LOCAL PACKAGE_:
  * document                  ✓
  * HTML image                ✓
  * stylesheet                ✓
  * CSS @import               ✓
  * nested font               ✓
  * nested CSS image          ✓
  * main.js execution         ✓

 _DENIED_:
  * http                      ✓
  * https                     ✓
  * ftp                       ✓
  * ws                        ✓
  * wss                       ✓
  * file                      ✓
  * store                     ✓
  * other asset package       ✓
  * literal .. root escape    ✓
  * encoded dot traversal     ✓
  * encoded slash traversal   ✓
  * double-encoded traversal  ✓

## Error Quality Matters

Do not collapse all failures into generic load failure.

Distinguish at least:

* `DeniedByPolicy`
* `UnsupportedScheme`
* `InvalidPath`
* `NotFound`
* `InvalidMime`
* `DecodeError`
* `IoError`

A missing local image is not the same kind of problem as a denied remote URL. A path traversal attempt is not the same kind of problem as a missing file.

Good errors are part of the runtime.

## Scheme Policy, v0

Allowed:

```text
asset://{active_package_id}/...
bundle://runtime/...
```

Denied:

```text
http://
https://
ws://
wss://
ftp://
file://
store:// as fetchable resource
asset:// for another package
asset:// paths escaping package root
```

Deferred:

```text
data:
blob:
```

Do not add new allowed schemes casually. New schemes should be documented in `/docs/local-runtime/resource-provider-plan.md`.

## Avoid These Defaults

Do not solve local runtime problems by adding:

* Node as the authority plane
* npm/package-manager sprawl
* localhost servers
* WebSocket IPC over `127.0.0.1`
* Electron-style main/renderer assumptions
* “just use a WebView” shortcuts
* remote fetch as a hidden fallback
* file-system access disguised as ordinary resource loading
* product/release/audience-driven architecture

A network-shaped internal bridge is still network-shaped. Avoid it unless there is a specific documented reason and no cleaner in-process path is available.

## Dependency and Authority Posture

Prefer:

* existing Servo machinery where it already does the hard browser-engine work
* small host-owned boundaries
* explicit resource/context objects
* compile-time feature removal where practical
* deterministic policy checks
* local package fixtures
* tests that prove denial behavior

Be suspicious of:

* ambient access
* hidden fallback behavior
* “temporary” remote loading
* APIs that silently grow authority
* changes that make debugging resource origins harder
* changes that require staying current with unrelated upstream browser features

This runtime does not need to be fresh. It needs to be understandable, preservable, and useful offline.

## Working Method

When starting a task:

1. Read the relevant section of `/docs/local-runtime/loader-map.md`.
2. Inspect Servo for the concrete crate/module path.
3. Add logging before changing behavior.
4. Record findings in `/docs/local-runtime/progress.md`.
5. Make the smallest change that moves one resource category toward host mediation.
6. Add or update a fixture/test when possible.
7. Update `/docs/local-runtime/loader-map.md` or `/docs/local-runtime/resource-provider-plan.md` with discovered facts.

When uncertain, prefer documenting the uncertainty over guessing.

## Definition of Useful Progress

Useful progress includes:

* identifying a Servo code path for a loader-map row
* adding request logging for a resource category
* separating `DeniedByPolicy` from `NotFound`
* proving a forbidden scheme cannot load
* routing a local package asset through the host provider
* documenting a hidden assumption
* reducing old network/resource coupling
* producing a reproducible test fixture

A small, well-documented seam is better than a broad unverified claim.

## Current Design Phrase

Use this phrase as the practical test:

```text
package-scoped, host-mediated offline document runtime
```

If a change supports that, it is probably aligned.

If a change turns the project back into a normal browser, Electron clone, localhost app server, or network-capable product shell, it is probably not aligned.

## Testing

Run focused tests or cargo checks only when they are expected
to complete in the agent environment.

Do not run broad Cargo checks for servo-script, servoshell,
or workspace-sized targets from a cold checkout. Record them
as deferred to the GitHub full-build receipt instead.
