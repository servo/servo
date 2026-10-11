/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A `CefMessageRouter`-style bridge for bidirectional native ↔ JavaScript
//! messaging.
//!
//! Native code registers handlers for messages sent from JavaScript via
//! `window.servoMessageRouter.postMessage(name, data)`, and can send messages
//! to JavaScript via [`crate::WebView::post_message`], which dispatches a
//! `servo-message` [`CustomEvent`] on `window`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use embedder_traits::JSValue;

/// A registered native handler for JS → native messages.
type MessageHandler = Rc<dyn Fn(&str, &str)>;

#[derive(Default)]
struct MessageRouterInner {
    /// Handlers for JS → native messages, keyed by message name.
    native_handlers: HashMap<String, Vec<MessageHandler>>,
    /// Messages received from JavaScript before a handler was registered.
    pending_js_messages: Vec<(String, String)>,
}

/// The message router for bidirectional native ↔ JavaScript communication.
///
/// Cloning a `MessageRouter` produces another handle to the same underlying
/// router state.
#[derive(Clone, Default)]
pub struct MessageRouter(Rc<RefCell<MessageRouterInner>>);

impl MessageRouter {
    /// Create a new, empty message router.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a handler for messages from JavaScript (JS → native).
    ///
    /// When JavaScript calls `window.servoMessageRouter.postMessage(name, data)`,
    /// `callback` is invoked with the message name and its data serialized as a
    /// JSON string. Multiple handlers may be registered for the same name; they
    /// are invoked in registration order.
    ///
    /// Any messages that were received before a handler was registered are
    /// replayed to the newly registered handler.
    pub fn add_message_handler<F>(&self, name: String, callback: F)
    where
        F: Fn(&str, &str) + 'static,
    {
        let handler: MessageHandler = Rc::new(callback);

        let pending = {
            let mut inner = self.0.borrow_mut();
            inner
                .native_handlers
                .entry(name.clone())
                .or_default()
                .push(handler.clone());

            let mut pending = Vec::new();
            let mut index = 0;
            while index < inner.pending_js_messages.len() {
                if inner.pending_js_messages[index].0 == name {
                    pending.push(inner.pending_js_messages.remove(index));
                } else {
                    index += 1;
                }
            }
            pending
        };

        for (message_name, data_json) in pending {
            handler(&message_name, &data_json);
        }
    }

    /// Remove all handlers registered for `name`.
    pub fn remove_message_handler(&self, name: &str) {
        self.0.borrow_mut().native_handlers.remove(name);
    }

    /// Whether any handlers are currently registered.
    pub fn has_handlers(&self) -> bool {
        !self.0.borrow().native_handlers.is_empty()
    }

    /// Dispatch a message received from JavaScript to the registered handlers.
    ///
    /// If no handler is registered for `name`, the message is queued and
    /// replayed when a handler is later registered.
    pub fn handle_js_message(&self, name: String, data_json: String) {
        let handlers = self.0.borrow().native_handlers.get(&name).cloned();
        match handlers {
            Some(handlers) => {
                for handler in handlers {
                    handler(&name, &data_json);
                }
            },
            None => self
                .0
                .borrow_mut()
                .pending_js_messages
                .push((name, data_json)),
        }
    }

    /// The JavaScript bootstrap that creates `window.servoMessageRouter`.
    ///
    /// This must be injected into every page (see
    /// [`crate::UserContentManager::inject_message_router`]) before messages
    /// can be exchanged.
    pub fn bootstrap_script() -> String {
        r#"
(function() {
    'use strict';

    if (window.servoMessageRouter) {
        return;
    }

    var servoMessageRouter = {
        _listeners: {},
        _outgoingQueue: [],
        _messageId: 0,

        // Send a message from JavaScript to native code.
        postMessage: function(name, data) {
            this._outgoingQueue.push({ name: String(name), data: data, id: ++this._messageId });
        },

        // Register a listener for messages from native code.
        addEventListener: function(name, handler) {
            if (!this._listeners[name]) {
                this._listeners[name] = [];
            }
            this._listeners[name].push(handler);
        },

        // Remove a previously registered listener.
        removeEventListener: function(name, handler) {
            var listeners = this._listeners[name];
            if (listeners) {
                var index = listeners.indexOf(handler);
                if (index !== -1) {
                    listeners.splice(index, 1);
                }
            }
        },

        // Internal: dispatch a message from native code to JS listeners.
        _dispatch: function(name, data) {
            var listeners = this._listeners[name];
            if (listeners) {
                listeners.forEach(function(handler) {
                    try {
                        handler(data);
                    } catch (e) {
                        console.error('servoMessageRouter handler error:', e);
                    }
                });
            }
        },

        // Internal: atomically take the outgoing queue (called by native code).
        _getOutgoingMessages: function() {
            var messages = this._outgoingQueue;
            this._outgoingQueue = [];
            return messages;
        }
    };

    window.servoMessageRouter = servoMessageRouter;
})();
"#
        .to_string()
    }

    /// The JavaScript expression used to poll for JS → native messages.
    pub fn poll_script() -> String {
        "window.servoMessageRouter ? window.servoMessageRouter._getOutgoingMessages() : []".to_string()
    }

    /// Build a JavaScript snippet that dispatches a native → JS message.
    pub fn message_to_javascript(name: &str, data_json: &str) -> String {
        format!(
            r#"(function() {{
                if (!window.servoMessageRouter) {{ return; }}
                window.servoMessageRouter._dispatch({name}, {data});
            }})();"#,
            name = json_string(name),
            data = data_json
        )
    }
}

impl std::fmt::Debug for MessageRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageRouter")
            .field("has_handlers", &self.has_handlers())
            .finish()
    }
}

/// Serialize a [`JSValue`] to a JSON string.
pub(crate) fn jsvalue_to_json_string(value: &JSValue) -> String {
    let mut output = String::new();
    write_json_value(value, &mut output);
    output
}

fn write_json_value(value: &JSValue, output: &mut String) {
    match value {
        JSValue::Undefined | JSValue::Null => output.push_str("null"),
        JSValue::Boolean(value) => output.push_str(if *value { "true" } else { "false" }),
        JSValue::Number(value) if value.is_finite() => output.push_str(&value.to_string()),
        JSValue::Number(_) => output.push_str("null"),
        JSValue::String(value) => write_json_string(value, output),
        JSValue::Array(items) => {
            output.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_json_value(item, output);
            }
            output.push(']');
        },
        JSValue::Object(entries) => {
            output.push('{');
            for (index, (key, value)) in entries.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_json_string(key, output);
                output.push(':');
                write_json_value(value, output);
            }
            output.push('}');
        },
        // Element, ShadowRoot, Frame and Window handles are not serializable.
        JSValue::Element(_) | JSValue::ShadowRoot(_) | JSValue::Frame(_) | JSValue::Window(_) => {
            output.push_str("null")
        },
    }
}

fn json_string(value: &str) -> String {
    let mut output = String::new();
    write_json_string(value, &mut output);
    output
}

fn write_json_string(value: &str, output: &mut String) {
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if (character as u32) < 0x20 => {
                output.push_str(&format!("\\u{:04x}", character as u32));
            },
            character => output.push(character),
        }
    }
    output.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_script_contains_api() {
        let script = MessageRouter::bootstrap_script();
        assert!(script.contains("servoMessageRouter"));
        assert!(script.contains("postMessage"));
        assert!(script.contains("addEventListener"));
        assert!(script.contains("removeEventListener"));
        assert!(script.contains("_getOutgoingMessages"));
    }

    #[test]
    fn test_message_roundtrip_and_pending() {
        let router = MessageRouter::new();

        // Message before handler registration is queued.
        router.handle_js_message("greet".to_string(), "\"hello\"".to_string());

        let received = Rc::new(RefCell::new(Vec::new()));
        let received_clone = received.clone();
        router.add_message_handler("greet".to_string(), move |name, data| {
            received_clone
                .borrow_mut()
                .push(format!("{name}:{data}"));
        });

        // The queued message is replayed.
        assert_eq!(
            received.borrow().as_slice(),
            ["greet:\"hello\"".to_string()]
        );

        router.handle_js_message("greet".to_string(), "\"again\"".to_string());
        assert_eq!(received.borrow().len(), 2);

        router.remove_message_handler("greet");
        assert!(!router.has_handlers());
    }
}
