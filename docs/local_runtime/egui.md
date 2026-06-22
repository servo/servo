# `--no-egui` ownership and content-service audit

Date: 2026-06-22

This note maps what the current desktop `--no-egui` switch removes.  The
important finding is that the switch is not only a shell-chrome switch: it also
short-circuits multiple page-originated embedder controls whose request/result
callbacks are needed for ordinary document behavior.

## Executive summary

`--no-egui` is parsed as `ServoShellPreferences::no_egui` and causes
`HeadedWindow::new` to store `gui: None` instead of constructing `Gui`.  That is
reasonable for persistent shell chrome.  The problem is that `HeadedWindow`
currently uses `self.gui.is_none()` as a proxy for "no user-interface service is
available" in several `PlatformWindow` delegate methods.  When the proxy is
false, content controls are immediately dismissed, denied, or default-submitted
without giving the user an opportunity to respond.

The broad gate has two effects:

* Persistent shell UI is removed as intended: URL bar, tabs, toolbar buttons,
  status tooltip, favicon texture upload, egui-specific accessibility grafts,
  and shell chrome event capture.
* Required content services are also removed: `<input type=file>`, `<select>`,
  `<input type=color>`, context menus, JavaScript dialogs, permissions,
  Bluetooth selection, and HTTP authentication dialogs.  IME is the notable
  exception: `--no-egui` still calls the platform IME path directly.

The smallest safe first fix for file input is to split file-picker request/result
handling from egui rendering.  In no-egui mode, invoke a platform-native file
picker (or a host-provided picker delegate) from `show_embedder_control` for
`EmbedderControl::FilePicker` and keep using `FilePicker::select/submit/dismiss`
so the existing FileManager callback path remains unchanged.  Do not route file
input through the current egui `Dialog::File` update loop.

## Current `--no-egui` control flow

1. `--no-egui` is parsed in `ports/servoshell/prefs.rs` as a headed desktop mode
   that does not construct egui chrome or presentation.
2. `HeadedWindow::new` reads the preference.  If it is true, the winit window is
   made visible and `gui` is set to `None`.  Otherwise `Gui::new(...)` is
   constructed.
3. Paint and input paths branch on `self.gui`:
   * With egui, redraw calls `Gui::update` and `Gui::paint`.
   * Without egui, redraw calls `paint_bare_frame`, which resizes the active
     WebView to the whole native window, repaints WebViews, blits the offscreen
     Servo render target into the window render target, and presents directly.
   * Mouse and keyboard events are forwarded to Servo when egui does not consume
     them.  With no egui, the forwarding path remains active because there is no
     egui consumer.
4. Content-originated controls enter the embedder through two main routes:
   * `EmbedderMsg::ShowEmbedderControl` for select, color, IME, and context
     menus, eventually reaching `HeadedWindow::show_embedder_control`.
   * `FileManagerThreadMsg::SelectFiles` for file input.  The net file manager
     asks the embedder for paths, `Servo::handle_net_embedder_message` wraps the
     request as `EmbedderControl::FilePicker`, and the same
     `HeadedWindow::show_embedder_control` delegate method handles it.
5. `show_embedder_control` currently begins with:

   ```text
   if self.gui.is_none() {
       self.dismiss_embedder_control_without_gui(embedder_control);
       return;
   }
   ```

   That single check suppresses all non-IME `EmbedderControl` variants in
   no-egui mode.

## Inventory table

| Feature / behavior | Category | Implementation and request/result path | Connection to egui | What `--no-egui` currently suppresses | Recommended remedy | Difficulty | Risk / dependencies |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Persistent toolbar, URL bar, navigation buttons, tab strip, location entry | Shell/diagnostic | `Gui::update` builds tab and toolbar UI; `HeadedWindow::new` only constructs `Gui` when `no_egui` is false. Toolbar height is subtracted from WebView coordinates and viewport sizing when present. | egui renders controls, owns shell input focus, tracks toolbar rect, favicon textures, and repaint requests. | Entire persistent shell chrome is gone, and the bare WebView fills the native window. | Keep suppressed in document-only mode. Any navigation capability should become a host command/API, not persistent chrome. | Low | Mostly viewport sizing and input coordinates. Current no-egui path already uses zero toolbar height. |
| WebRender debug toggles, sampling profiler shortcut, capture, page zoom/reload/back/forward shortcuts, tab shortcuts | Mixed | `HeadedWindow::handle_intercepted_key_bindings` runs before page delivery for non-overridable shortcuts; `notify_input_event_handled` runs for overridable shortcuts after Servo reports whether page handled the key. | Not rendered by egui; lives in `HeadedWindow` keyboard routing. Conceptually shell/diagnostic despite not depending on `Gui`. | Not suppressed by `--no-egui`; keyboard shortcuts still work because input routing remains active. | Audit separately. For local runtime, keep page editing shortcuts but disable shell/debug/navigation shortcuts under a runtime policy flag, not under egui rendering. | Medium | Changes can affect input routing and web-content default prevention. Must preserve cut/copy/paste and IME behavior. |
| Status text tooltip and favicon texture upload | Shell/diagnostic | `EmbedderMsg::Status` and `NewFavicon` update `WebView` state; `Gui::update` renders status tooltip and loads favicon textures. | egui renders status tooltip and owns favicon GPU texture cache. | Visual status/favicons disappear; underlying WebView state may still update. | Keep suppressed with shell chrome. For diagnostics, log instead of showing persistent UI. | Low | Repaint scheduling for favicon texture loads currently mentions egui; removing visual chrome should not affect content rendering. |
| `<input type=file>` native file chooser | Required page behavior | DOM creates `EmbedderControlRequest::FilePicker`; `DocumentEmbedderControls::show_embedder_control` sends `FileManagerThreadMsg::SelectFiles` to the resource/file-manager thread; `FileManager::select_files` sends `NetToEmbedderMsg::SelectFiles`; `Servo::handle_net_embedder_message` wraps it as `EmbedderControl::FilePicker`; `HeadedWindow::show_embedder_control` creates `Dialog::File`; `Dialog::update` starts `egui_file_dialog`, calls `FilePicker::select`, then `submit`/`dismiss`; file manager turns returned paths into `SelectedFile` entries and sends `EmbedderControlResponse::FilePicker`; script handles the response on the main thread. | egui currently owns the visible picker via `egui_file_dialog` and, more importantly, the polling/update loop that notices picked/cancelled state and calls `FilePicker::submit` or `dismiss`. The actual file-manager metadata path is outside egui. | `dismiss_embedder_control_without_gui` immediately calls `FilePicker::dismiss`, so no picker opens and the DOM receives cancellation/no files. | Move request/result handling outside the `self.gui.is_none()` gate. Use a platform-native file dialog or host picker delegate in no-egui mode, preserving `FilePicker::select/submit/dismiss` and the existing FileManager callback path. Do not keep full egui alive invisibly. | Medium | Must integrate with event loop without blocking indefinitely. File dialog return must wake repaint/update only if needed, and must not break the async FileManager oneshot. Current egui implementation requires repeated frame updates; native picker should not. |
| HTML `<select>` dropdown | Required page behavior | `DocumentEmbedderControls` sends `EmbedderMsg::ShowEmbedderControl` with `EmbedderControlRequest::SelectElement`; Servo wraps it as `EmbedderControl::SelectElement`; `HeadedWindow::show_embedder_control` creates `Dialog::SelectElement`; `Dialog::update` renders options near the element, mutates selected option ids with `SelectElement::select`, and calls `submit`; response returns through `EmbedderToConstellationMessage::EmbedderControlResponse` to `HTMLSelectElement::handle_embedder_response`. | egui renders the transient popup, performs option hit-testing and keyboard escape handling, owns backdrop dismissal, and dispatches selection via `SelectElement::submit`. | `dismiss_embedder_control_without_gui` immediately calls `prompt.submit()` with the existing selection, so the popup never appears and the user cannot change selection. | Retain a minimal transient-widget layer or replace with a Servo/platform select popup. This should be explicitly separate from shell chrome. Short-term: a minimal content-popup renderer may still use egui, but it should be named and gated as content transient UI, not `Gui`. | Medium/High | Affects pointer capture, positioning, focus, backdrop clicks, keyboard dismissal, and repaint scheduling. Multi-select requires stateful transient UI. |
| `<input type=color>` picker | Required page behavior | `EmbedderControlRequest::ColorPicker` follows the same ShowEmbedderControl path; `HeadedWindow` creates `Dialog::ColorPicker`; egui color widget updates `ColorPicker::select` and closes. | egui renders color widget, owns current color state and selection/dismiss buttons. | No UI opens; `dismiss_embedder_control_without_gui` selects `None` and submits, effectively defaulting/dismissing. | Prefer platform-native color picker where available, or a minimal transient content-widget layer. Keep result callback outside shell chrome. | Medium | Needs asynchronous or modal return handling and correct color/default semantics. |
| Text input IME / virtual keyboard positioning | Required page behavior | `EmbedderControlRequest::InputMethod` goes to `HeadedWindow::show_embedder_control`; with egui it calls `show_ime`; without egui `dismiss_embedder_control_without_gui` also calls `show_ime`. `show_ime` enables winit IME and sets cursor area, offset by toolbar height. | Not egui-rendered. It uses winit/platform IME. egui only affects toolbar offset and event consumption. | Not suppressed. This is the good model: content service remains active without egui chrome. | Keep outside any egui gate. Use this as precedent for other content services. | Low | Coordinate correctness depends on toolbar offset; no-egui offset is zero. |
| Context menu from right-clicking web content | Mixed, but content-facing portions are required | `DocumentEmbedderControls::show_context_menu` hit-tests anchor/image/editable text, builds context-sensitive items, sends `EmbedderControlRequest::ContextMenu`; `Dialog::ContextMenu` renders popup and sends selected `ContextMenuAction`; response handler performs history navigation, reload, open link/image in new WebView, clipboard writes, editing actions, or select-all. | egui renders transient menu, handles outside clicks, enables/disables items, and dispatches chosen action. The action semantics live in script/document code, not egui. | Immediate `context_menu.dismiss()`, so no context actions are available. | Split menu model into content-required editing/link actions and shell/browser actions. Keep or replace a minimal transient menu for edit/copy/paste/select-all and maybe copy link/image. Deny or host-mediate open-in-new-view/history/reload in local runtime. | High | Mixed authority: menu can trigger navigation, clipboard, and editing. Requires policy decisions and careful event/focus handling. |
| JavaScript `alert`, `confirm`, `prompt` | Required page behavior, though host-policy-sensitive | `Window::alert/confirm/prompt` send `EmbedderMsg::ShowSimpleDialog`; Servo wraps as `EmbedderControl::SimpleDialog`; `Dialog::{Alert,Confirm,Prompt}` renders modal egui dialogs and sends responses. Drop/default behavior returns OK for alert and Cancel for confirm/prompt. | egui renders modal dialogs and owns text entry for prompt. Response objects are independent wrappers with drop defaults. | `dismiss_embedder_control_without_gui` confirms alerts and dismisses confirm/prompt immediately; scripts cannot present prompts or receive user input. | Provide a minimal content dialog service independent of shell chrome. For offline runtime, consider host policy for whether blocking JS dialogs are allowed, but do not conflate that with no-egui. | Medium | JS dialogs are synchronous/user-prompt-sensitive and can block script progress. Need event-loop-safe modality and anti-spoofing presentation. |
| Permission prompts | Required host service when APIs are enabled | `EmbedderMsg::PromptPermission` and `RequestWakeLockPermission` become `PermissionRequest`; `HeadedWindow::show_permission_dialog` creates `Dialog::Permission`; egui renders Allow/Deny buttons. | egui renders prompt. The default `AllowOrDenyRequest` fallback is Deny. | No prompt; no-egui immediately denies. | For local runtime, keep default deny for disabled capabilities, but route allowed/askable capabilities through a host permission service outside egui. | Medium | Permission decisions affect security model and promise resolution. Must remain deterministic and logged. |
| Bluetooth device selection | Required only if Web Bluetooth remains enabled; otherwise policy-deny | `components/bluetooth` sends `EmbedderMsg::GetSelectedBluetoothDevice`; `HeadedWindow::show_bluetooth_device_dialog` creates `Dialog::SelectDevice`; egui renders ComboBox and sends selected address/cancel. | egui renders device picker and owns selected index. | No prompt; request is cancelled. | For local offline runtime, likely deny/disable Web Bluetooth. If retained, move to host capability service outside shell chrome. | Medium | External device authority; async callback. Should be capability-gated. |
| HTTP authentication dialog | Required for network-capable browsing, likely not for offline runtime | Authentication requests are wrapped as `AuthenticationRequest`; `HeadedWindow::show_http_authentication_dialog` creates `Dialog::Authentication`; egui renders username/password fields. | egui renders credentials form and owns typed state. | No prompt; request is dropped/cancelled. | In local-runtime package mode, remote HTTP should be denied before auth. For general servoshell no-egui, provide native/minimal auth dialog or explicit cancellation policy. | Medium | Security-sensitive credentials; event-loop modality. |
| Clipboard API and editing actions | Required page behavior, but permission-sensitive | Clipboard messages (`ClearClipboard`, `GetClipboardText`, `SetClipboardText`) go through `Servo::handle_embedder_message` to the WebView clipboard delegate; keyboard cut/copy/paste shortcuts are handled in `HeadedWindow`; context-menu clipboard actions depend on the egui context menu. | Direct clipboard delegate path does not require egui. Context-menu UI does. | Clipboard APIs and keyboard editing are not directly suppressed by no-egui; context-menu access to clipboard/editing actions is suppressed. | Keep delegate path independent of egui. Add explicit permission/policy prompts outside egui if required. Restore content context menu separately. | Medium | Clipboard authority and user activation requirements. |
| Notifications | Required only if notifications are enabled; host service | `EmbedderMsg::ShowNotification` routes to WebView/global delegate. No obvious egui dialog in this path. | Not egui-owned in the inspected desktop path. | Not directly suppressed by the `show_embedder_control` no-egui gate. | Keep as host service with local-runtime capability policy. | Low/Medium | Platform notification authority and permissions. |
| DevTools connection prompt | Shell/diagnostic | `RequestDevtoolsConnection` goes to top-level delegate, not `HeadedWindow` egui dialog in the inspected path. | Not part of `Dialog`; may be handled by shell delegate elsewhere. | Not directly covered by the no-egui content-control gate. | Keep disabled/denied for document-only runtime unless explicitly enabled for diagnostics. | Low | Debug authority and network/listener exposure. |
| AccessKit / accessibility tree grafting | Mixed | `Gui::new` sets up egui/accesskit integration; `Gui::update` grafts WebView accesskit tree ids into egui; `HeadedWindow::handle_winit_app_event` still toggles accessibility active for accesskit events, and only forwards to egui when `gui` exists. | egui owns AccessKit adapter and tree update draining. | With no egui, egui adapter updates are gone; bare WebView accessibility forwarding is incomplete in this path. | Treat separately from shell chrome. If document-only runtime needs accessibility, provide a non-egui AccessKit bridge for WebView content. | High | Platform accessibility event loop, tree IDs, action forwarding. |

## Evidence by subsystem

### Construction and broad gate

The relevant state is a single optional `Gui` field on `HeadedWindow`.  In
`HeadedWindow::new`, no-egui makes the native window visible and leaves `gui` as
`None`; the egui path constructs `Gui::new`.  Later, `show_embedder_control`,
`show_bluetooth_device_dialog`, `show_permission_dialog`, and
`show_http_authentication_dialog` all treat `gui.is_none()` as a reason to auto
cancel/deny/drop page-originated UI.

That means the current switch name is effectively interpreted as "no egui and no
content dialogs", not merely "no persistent shell chrome".

### File input path

`<input type=file>` is already mostly independent of egui.  The FileManager owns
validation and creation of `SelectedFile` metadata.  The embedder only needs to
return paths or cancellation.

The only egui-specific part is the current desktop path for obtaining those
paths:

* `Dialog::new_file_dialog` configures `egui_file_dialog` filters from
  `FilePicker::filter_patterns`.
* `Dialog::update` starts either `pick_file` or `pick_multiple`, polls
  `dialog.update(ctx).state()`, and calls `FilePicker::select` plus
  `submit`/`dismiss`.

In no-egui mode, this polling path is never installed, because the
`EmbedderControl::FilePicker` request is dismissed before a dialog is created.
This explains the observed file picker disappearance.

### Select/dropdown path

`<select>` popups are not browser chrome in the current architecture; they are
`EmbedderControlRequest::SelectElement` requests created by the active document.
The egui dialog renders the transient option list, updates the selected ids, and
submits them.  The no-egui fallback calls `submit()` immediately with the old
selection, which makes the control look inert from the page user's point of
view.

This explains the observed disappearance of ordinary page dropdowns.

### Other content-originated popup/dialog paths

The same pattern applies to color picker, context menu, JavaScript simple
dialogs, permission prompts, Bluetooth device selection, and HTTP auth.  Some of
these are not required for the local runtime's first milestone, and some should
be denied by policy, but their policy should be explicit and logged.  They should
not disappear solely because persistent shell chrome is disabled.

### Work currently performed only inside an egui frame/update call

`Gui::update` is the only place that calls `headed_window.for_each_active_dialog`
and therefore the only place that calls `Dialog::update`.  For egui-backed
transient controls, request processing, UI rendering, hit-testing, escape/outside
click handling, and result dispatch all happen during an egui frame.  Without a
`Gui`, the dialog collection is not serviced at all; current no-egui avoids a
stalled collection by immediately dismissing controls.

Any fix that keeps egui for temporary transient controls must introduce a
separate update/pump path for those controls, not rely on full `Gui::update`.
However, the preferred direction is to replace the file picker first with a
native/host path that does not require an egui frame.

## Smallest safe first fix to restore file input behavior

Implement a no-egui file-picker path only:

1. In `HeadedWindow::show_embedder_control`, handle
   `EmbedderControl::FilePicker(file_picker)` before the `self.gui.is_none()`
   fallback.
2. If `gui` exists, keep the current `Dialog::new_file_dialog(file_picker)` path.
3. If `gui` is absent, invoke a platform-native or host-supplied picker using the
   `FilePickerRequest` data exposed through `FilePicker` methods:
   * existing selected/current paths for initial directory or preselection where
     platform support exists;
   * `filter_patterns` for file filters;
   * `allow_select_multiple` for single vs. multi selection.
4. On success, call `file_picker.select(&paths)` and `file_picker.submit()`.
   On cancellation or error, call `file_picker.dismiss()`.
5. Leave FileManager validation unchanged so selected paths still become
   `SelectedFile` records through the existing file-manager response path.
6. If the native picker must run asynchronously, send the result back onto the
   application event loop and complete the `FilePicker` there.  If it is a
   short-lived platform-modal dialog, document that it blocks the window while
   open and verify the FileManager oneshot still resolves.

This is the smallest fix because it does not alter DOM control IDs, the
FileManager thread, `SelectedFile` metadata creation, WebDriver testing bypass,
or the response handling in script.  It only replaces the missing path-provider
piece in no-egui mode.

## Recommended staged plan

### Stage 1: Rename the conceptual gates

Introduce terminology and structure that separates:

* `shell_chrome`: persistent toolbar, tabs, diagnostics, status UI, favicon UI.
* `content_transient_ui`: page-originated controls such as select, file, color,
  context menu, JS dialogs, permission prompts, and IME.
* `host_capability_prompts`: permission/device/clipboard/auth prompts that may be
  denied or delegated by runtime policy.

Do not use `gui.is_none()` as a general proxy for all three.

### Stage 2: Restore file input with a non-egui provider

Land the minimal file-picker fix described above.  This immediately proves the
separation: no persistent egui shell, but a page-originated host service still
works and returns through Servo's existing FileManager callback chain.

### Stage 3: Extract a transient content-control service

Create an internal trait or module boundary for content controls, for example:

```text
ContentUiService
  show_file_picker(FilePicker)
  show_select(SelectElement)
  show_color_picker(ColorPicker)
  show_context_menu(ContextMenu)
  show_simple_dialog(SimpleDialog)
  show_permission(PermissionRequest)
```

Provide implementations:

* `EguiContentUiService` for the current full egui mode.
* `NoShellContentUiService` for no-egui mode, initially with native file picker,
  platform IME, explicit deny/default behavior for unsupported prompts, and logs.

The service should be owned by `HeadedWindow` separately from `Gui`.

### Stage 4: Replace or minimize egui transient widgets

For each content control:

1. File picker: platform-native/host-owned path.  Replacement target, not egui.
2. Select dropdown: either Servo-owned popup rendering/input or a minimal
   transient-widget layer.  If egui remains temporarily, name it as a content
   transient dependency and pump only that layer.
3. Color picker: platform-native where available; otherwise minimal transient
   widget.
4. JS dialogs and permissions: host-owned modal/prompt service with local-runtime
   policy and anti-spoofing presentation.
5. Context menu: split editing/copy actions from browser/navigation actions and
   mediate external authority.

### Stage 5: Policy and logging for denied host services

For the package-scoped local runtime, denied services should produce explicit
runtime logs rather than silent no-egui cancellations.  Examples:

* permission prompt denied because capability is unavailable;
* Bluetooth denied because external device authority is disabled;
* HTTP auth unavailable because remote network is denied;
* context-menu open-in-new-view denied because cross-document navigation is not
  granted.

This keeps the no-egui shell goal aligned with the project phrase:
`package-scoped, host-mediated offline document runtime`.
