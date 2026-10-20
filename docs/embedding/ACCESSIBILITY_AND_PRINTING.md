# Accessibility, Printing, and Credential Management Status

## 1. Accessibility (AccessKit Integration)

Servo builds an [AccessKit](https://accesskit.dev/) tree for each page. It is an
in-process tree, not a platform adapter: Servo produces `accesskit::TreeUpdate`s and the
**embedder** owns the adapter that turns them into UI Automation, NSAccessibility or
AT-SPI objects. `components/layout/accessibility_tree.rs` builds the nodes (roles, states,
bounds) from the layout DOM, `components/script` drives activation and action dispatch,
and `components/servo/webview.rs` grafts each document's tree under a per-`WebView` root
node, so a host with several WebViews can keep them in one adapter.

- **DOM to A11y Tree:** layout updates AccessKit nodes as the DOM changes, with ARIA
  roles, states and bounding rects taken from the DOM nodes' `AccessibilityInformation`.
- **Activation:** off by default. The `accessibility_enabled` preference must be `true`,
  and the embedder must then activate the `WebView`: `WebView::set_accessibility_active`
  in Rust, `servo_webview_set_accessibility_active` in C. Inactive documents send nothing,
  deliberately, so a page pays no cost unless a screen reader is present.
- **Action Dispatch:** `Servo::forward_accessibility_action` routes an
  `accesskit::ActionRequest` to the document that owns the target node, where it becomes a
  DOM event (`Click`, `Focus`, `ScrollIntoView`, ...).
- **C Embedding Exposure (W6.4):** the C API delivers updates through the
  `notify_accessibility_tree_update` delegate field; the payload is a `ServoValue`
  dictionary in AccessKit's own JSON schema, matching CEF's `CefValue`. It is exercised
  end to end by `test_accessibility_tree_update` in `tests/capi/c/test_integration.c`,
  which turns the preference and the activation on, loads a page, and waits for an update
  carrying nodes. Forwarding an action from C is **not** exposed yet (see the build plan's
  §D.2.1).

> **Correction (W6.4).** This section previously read "Embedders can hook `AccessKit`
> updates via delegate notifications or by querying the root `AccessKit` adapter handle",
> and located the integration in `components/script/dom/node.rs`. Both claims were wrong:
> there is no adapter handle to query, because the adapter belongs to the embedder, and
> until W6.4 the C API had no accessibility surface at all.

---

## 2. Printing and PDF Export

CEF exposes `CefBrowserHost::PrintToPDF`.

### 2.1 Feasibility Assessment (Dated: 2026-10-20)
- **WebRender Vector vs Raster Pipeline:** WebRender's display lists contain vector primitives, clips, text runs, and image keys. While directing raw WebRender commands to a vector recording backend (such as Skia PDF or Cairo) provides scalable text, it requires paginated layout re-flows (`@media print` pagination).
- **Embedded PDF Generation:** For embedders and full consumer browser printing, `servo_webview_print_to_pdf` generates a compliant PDF container (`%PDF-1.4`) with page catalog, dimensions (Letter/A4), margins, and orientation settings.
- **C API Exposure (W6.5):** Implemented in `ffi/capi/printing.rs`:
  - `ServoPdfPrintSettings` controls page width, height, margins, scale, background printing, and landscape flags.
  - `servo_webview_print_to_pdf` writes to the target file path and reports completion via `ServoPdfPrintCallback`.
  - Exercised end-to-end by `test_webview_print_to_pdf` in `tests/capi/c/test_api.c`, verifying non-null argument checks, file generation, and `%PDF` magic header validation.

---

## 3. Credential Management & Autofill

- **W3C Credential Management:** WebAuthn (`navigator.credentials.create`, `get`) is partially prototyped in `components/script/dom/credentialmanagement/`.
- **Form Autofill:** When input fields with `autocomplete` attributes receive focus, Constellation dispatches `ShowEmbedderControl::AutofillSuggestion` allowing the host application's password manager or credential store to provide suggestion overlays.
