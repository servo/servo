# Native <-> JavaScript Message Router Architecture (CEF / WebKit Parity)

## 1. Executive Summary

A critical capability required by modern web embedders (such as CEF's `CefMessageRouterBrowserSide`, WebKit's `WKScriptMessageHandler`, and WebView2's `CoreWebView2.WebMessageReceived`) is bidirectional, asynchronous message passing between host native code and in-page JavaScript execution contexts.

Previously, Servo offered script evaluation (`evaluate_javascript`) and user script injection (`UserContentManager`), but lacked a dedicated, asynchronous, bidirectional message router.

This document specifies the message router architecture, message flow, DOM interfaces, and C FFI API.

---

## 2. Architectural Overview

```
+-------------------------------------------------------------+
|                      Host Application (C / Rust)            |
|                                                             |
|   servo_webview_add_script_message_handler("bridge", cb)    |
|   servo_webview_post_message("{\"status\": \"ok\"}")        |
+------------------------------+------------------------------+
                               |
                   EmbedderMsg / C Delegate
                               |
+------------------------------v------------------------------+
|                         Constellation                       |
|                                                             |
|   - Routes messages between WebViewId and PipelineId        |
|   - Origin security check and handler registration map       |
+------------------------------+------------------------------+
                               |
                    IPC Channel (ScriptMsg)
                               |
+------------------------------v------------------------------+
|                    Content Process (ScriptThread)           |
|                                                             |
|   - Injected window.servo object:                           |
|       * window.servo.postMessage(handler, message)          |
|       * window.servo.addEventListener("message", fn)        |
+-------------------------------------------------------------+
```

---

## 3. JavaScript Interface (`window.servo`)

In the page execution environment, an object `window.servo` is exposed to frames authorized by the embedder:

### 3.1 JS -> Native (Host Dispatch)
```javascript
// Post message to a named native handler:
window.servo.postMessage("nativeBridge", {
    action: "getUserData",
    userId: 42
});
```
- Accepts primitives, objects, and strings.
- Automatically serialized using JSON or HTML StructuredClone algorithm.

### 3.2 Native -> JS (Event Listener)
```javascript
window.servo.addEventListener("message", (event) => {
    console.log("Received message from native host:", event.data);
});
```

---

## 4. C Embedding API

### 4.1 Handler Registration
```c
/**
 * Callback invoked when JavaScript in a WebView calls window.servo.postMessage.
 *
 * @param webview The WebView handle that sent the message.
 * @param handler_name The handler identifier passed from JavaScript.
 * @param message_json The JSON-serialized message payload (UTF-8).
 * @param user_data Embedder context passed at registration.
 */
typedef void (*ServoScriptMessageCallback)(
    ServoWebView* webview,
    const char* handler_name,
    const char* message_json,
    void* user_data
);

/**
 * Registers a named script message handler on a WebView.
 */
int servo_webview_add_script_message_handler(
    ServoWebView* webview,
    const char* handler_name,
    ServoScriptMessageCallback callback,
    void* user_data
);

/**
 * Removes a previously registered script message handler.
 */
int servo_webview_remove_script_message_handler(
    ServoWebView* webview,
    const char* handler_name
);

/**
 * Dispatches an asynchronous message from the native embedder to in-page JavaScript.
 * The message triggers window.servo message events.
 *
 * @param webview The WebView handle to post to.
 * @param message_json A UTF-8 encoded JSON string payload.
 */
int servo_webview_post_message(
    ServoWebView* webview,
    const char* message_json
);
```

---

## 5. Security & Threat Mitigation

1. **Origin Restriction:** Handlers can be restricted to specific allowed origins (`Option<HashSet<ImmutableOrigin>>`) to prevent untrusted third-party ads or iframes from messaging the native host.
2. **Payload Size Limits:** Max message payload is capped (e.g. 16MB) to protect against memory exhaustion attacks.
3. **Structured Clone vs. Raw Strings:** All messages undergo strict deserialization before invoking native callback handlers.
