# Accessibility, Printing, and Credential Management Status

## 1. Accessibility (AccessKit Integration)

Servo integrates **AccessKit** (`components/script/dom/node.rs` and `accesskit`) to expose semantic accessibility trees to operating system screen readers (UI Automation on Windows, NSAccessibility on macOS, AT-SPI on Linux):
- **DOM to A11y Tree:** DOM mutations update AccessKit `Node` trees with appropriate ARIA roles, states, and bounding rects.
- **Action Dispatch:** Host platform accessibility actions (Click, Focus, ScrollIntoView) are translated to DOM events.
- **C Embedding Exposure:** Embedders can hook `AccessKit` updates via delegate notifications or by querying the root `AccessKit` adapter handle.

---

## 2. Printing and PDF Export

CEF exposes `CefBrowserHost::PrintToPDF`. In Servo:
- **WebRender Vector / PDF Surface:** WebRender display lists can be directed to a vector recording backend (such as Cairo or Skia PDF) to generate paginated PDF documents.
- **Page Setup:** Print media styles (`@media print`) and page margins are calculated during print layout pagination.
- **C API Roadmap:** Expose `servo_webview_print_to_pdf(webview, print_settings, callback, user_data)`.

---

## 3. Credential Management & Autofill

- **W3C Credential Management:** WebAuthn (`navigator.credentials.create`, `get`) is partially prototyped in `components/script/dom/credentialmanagement/`.
- **Form Autofill:** When input fields with `autocomplete` attributes receive focus, Constellation dispatches `ShowEmbedderControl::AutofillSuggestion` allowing the host application's password manager or credential store to provide suggestion overlays.
