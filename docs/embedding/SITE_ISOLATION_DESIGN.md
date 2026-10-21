# Site and Origin Isolation Architecture in Servo

## 1. Executive Summary

Modern web engines rely on process-level sandboxing to protect sensitive user data (passwords, session cookies, local storage, private intranet endpoints) from side-channel attacks such as Spectre (CVE-2017-5753, CVE-2017-5715) and memory corruption exploits. 

While sandboxing confines what an exploited process can do against the host operating system, **Site and Origin Isolation** confines what an exploited renderer process can see within the web engine itself.

This document describes the architectural transition from Servo's current **site-keyed** process model to an **origin-keyed** process isolation model with Out-Of-Process Iframes (OOPIF).

---

## 2. Current Architecture: Site-Keyed Browsing Contexts

In Servo's constellation (`components/constellation/constellation.rs`), event loops and content processes are grouped within a `BrowsingContextGroup`:

```rust
pub(crate) struct BrowsingContextGroup {
    /// Indexed by registered domain name (eTLD+1 as a `Host`) to event loops.
    event_loops: HashMap<Host, Weak<EventLoop>>,
}
```

### 2.1 The Site-Keying Policy (eTLD+1)
When navigating to a new URL, `constellation.rs` calculates the registrable domain via `registered_domain_name(&url)` (e.g., `example.com` for `a.example.com` and `b.example.com`).

- **Historical Motivation:** The legacy DOM property `document.domain` permitted different subdomains of the same registrable domain to mutate `document.domain` to match their common suffix, thereby acquiring synchronous same-origin script access to each other's DOM trees and JavaScript heaps. To support synchronous DOM access, both documents had to share a single JavaScript thread and memory space.
- **Security Vulnerability:** Because `a.example.com` and `b.example.com` share a process, a Spectre vulnerability or heap exploit in `a.example.com` can read memory belonging to `b.example.com` (such as cookies, authentication tokens, or DOM state). This is acute for shared multi-tenant hosting platforms (e.g. `user1.github.io` vs `user2.github.io`).

---

## 3. Target Architecture: Origin-Keyed Process Isolation

Under the W3C / WHATWG HTML specification (§Agent Clusters and `Origin-Agent-Cluster`), browsers are moving to origin-keyed agent clusters by default:

```
+--------------------------------------------------------------+
|                         Constellation                        |
|                                                              |
|  [BrowsingContextGroup]                                      |
|    |                                                         |
|    +-- Key: ("https", "a.example.com", 443) -> Process A     |
|    |     |                                                   |
|    |     +-- Top Pipeline: https://a.example.com             |
|    |                                                         |
|    +-- Key: ("https", "b.example.com", 443) -> Process B     |
|    |     |                                                   |
|    |     +-- Cross-Origin OOPIF: https://b.example.com       |
|    |                                                         |
|    +-- Key: ("https", "bank.com", 443)      -> Process C     |
+--------------------------------------------------------------+
```

### 3.1 Comparison

| Dimension | Site-Keyed (Legacy) | Origin-Keyed (Target) |
| :--- | :--- | :--- |
| **Isolation Key** | Scheme + eTLD+1 (`example.com`) | Scheme + Host + Port (`https://sub.example.com:443`) |
| **`document.domain`** | Enabled by default; mutates origin | Deprecated; throws or no-op |
| **Spectre Defense** | Ineffective across subdomains | Complete process-boundary isolation |
| **Memory Overhead** | Lower (fewer processes) | Higher (more granular processes) |
| **Cross-Origin Iframes** | Shared process if same site | Out-Of-Process Iframes (OOPIF) |

---

## 4. Key Subsystem Requirements

### 4.1 Process Keying & Navigation
The constellation's `event_loops` lookup must use a dedicated `IsolationKey` enum:
```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ProcessIsolationKey {
    Site(Host),
    Origin(ImmutableOrigin),
    Opaque(OpaqueOriginId),
}
```
1. If origin isolation is configured or requested via `Origin-Agent-Cluster: ?1`, `ProcessIsolationKey::Origin` is used.
2. Sandboxed iframes without `allow-same-origin` receive a unique `ProcessIsolationKey::Opaque`.

### 4.2 Out-Of-Process Iframes (OOPIF)
When an iframe's origin differs from its embedding document:
1. **Script Boundary:** The parent frame and iframe cannot directly dereference each other's JavaScript pointers. All communication occurs via `window.postMessage`, mediated through the Constellation's IPC routing.
2. **WebRender Compositing:** The iframe renders into an offscreen pipeline surface. The parent process embeds an `IframeDisplayItem` referencing the child pipeline's `PipelineId`. WebRender aggregates the scene display lists in the GPU compositor.
3. **Event Hit-Testing:** Input events (mouse, touch, keyboard) arriving at the browser process are hit-tested against the composite frame tree. If the target coordinates fall inside a cross-process iframe, the event is translated to iframe-local coordinates and dispatched over IPC to the child content process.

### 4.3 `Origin-Agent-Cluster` Header Support
Per HTML spec:
```http
Origin-Agent-Cluster: ?1
```
When this HTTP response header is received on the initial navigation to an origin, Constellation marks that origin as dedicated. Subsequent navigations within that browsing context group must never place different origins in that event loop.

---

## 5. Phased Implementation Roadmap

- **Phase 1 (Landed):** Architectural specification and isolation key design.
- **Phase 2 (Landed):** Configurable origin isolation. Plumbed `opts.origin_isolation`, `--origin-isolation` CLI flag in servoshell, and C API `servo_options_set_origin_isolation(options, bool)` allowing embedders to enforce origin-keyed process pooling directly at the constellation level. Subdomains and origins are partitioned into isolated event loops.
- **Phase 3 (Header Support):** Wire `Origin-Agent-Cluster` response header parsing into the navigation response pipeline in constellation.
- **Phase 4 (Full Out-of-Process Iframes):** WebRender scene aggregation for cross-origin child pipelines with out-of-process hit-testing and input event routing over constellation IPC.
- **Phase 4 (OOPIF Compositing):** Enable cross-process pipeline embedding for non-same-origin `HTMLIFrameElement` nodes with asynchronous layout synchronization.
