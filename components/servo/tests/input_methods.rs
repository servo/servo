/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! WebView input method tests
mod common;

use std::rc::Rc;

use servo::{
    CompositionEvent, CompositionState, ImeEvent, InputEvent, JSValue, Key, KeyState,
    KeyboardEvent, WebView, WebViewBuilder,
};
use url::Url;

use crate::common::{
    ServoTest, WebViewDelegateImpl, evaluate_javascript,
    show_webview_and_wait_for_rendering_to_be_ready,
};

#[derive(Debug, Clone, PartialEq)]
enum InputMethodEvent {
    CompositionStart,
    CompositionUpdate(String),
    CompositionEnd(String),
    Input(String, Option<String>),
    BeforeInput(String, Option<String>),
}

struct InputMethodTest {
    servo_test: ServoTest,
    webview: WebView,
    delegate: Rc<WebViewDelegateImpl>,
}

impl InputMethodTest {
    fn new(body: &str) -> Self {
        let servo_test = ServoTest::new();
        let delegate = Rc::new(WebViewDelegateImpl::default());
        let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
            .delegate(delegate.clone())
            .url(Url::parse(format!("data:text/html,{body}").as_str()).unwrap())
            .build();
        show_webview_and_wait_for_rendering_to_be_ready(&servo_test, &webview, &delegate);
        evaluate_javascript(
            &servo_test,
            webview.clone(),
            "window.input_method_test_received = [];",
        )
        .expect("JS evaluation should not fail");
        Self {
            servo_test,
            webview,
            delegate,
        }
    }

    fn send_input_event(&self, event: InputEvent) {
        *self.delegate.input_event_result.borrow_mut() = None;
        self.webview.notify_input_event(event);
        self.servo_test
            .spin(move || self.delegate.input_event_result.borrow().is_none());
    }

    fn focus(&self, id: &str) {
        evaluate_javascript(
            &self.servo_test,
            self.webview.clone(),
            format!("document.getElementById('{id}').focus();"),
        )
        .expect("JS evaluation should not fail");
    }

    fn install_event_listeners(&self, id: &str) {
        for event in &["beforeinput", "input"] {
            evaluate_javascript(
                &self.servo_test,
                self.webview.clone(),
                format!("document.getElementById('{id}').addEventListener('{event}', (event) => {{ \
                        window.input_method_test_received.push({{ name: '{event}', inputType: event.inputType, data: event.data }}); \
                    }});"),
            ).expect("JS evaluation should not fail");
        }
        for event in &["compositionstart", "compositionupdate", "compositionend"] {
            evaluate_javascript(
                &self.servo_test,
                self.webview.clone(),
                format!("document.getElementById('{id}').addEventListener('{event}', (event) => {{ \
                        window.input_method_test_received.push({{ name: '{event}', data: event.data }}); \
                    }});"),
            ).expect("JS evaluation should not fail");
        }
    }

    fn get_text_control_value(&self, id: &str) -> String {
        let value = evaluate_javascript(
            &self.servo_test,
            self.webview.clone(),
            format!("document.getElementById('{id}').value"),
        );
        let Ok(JSValue::String(content)) = value else {
            panic!("Should get a string from JS");
        };
        content
    }

    fn drain_received_events(&self) -> Vec<InputMethodEvent> {
        let raw_events = evaluate_javascript(
            &self.servo_test,
            self.webview.clone(),
            "let received = window.input_method_test_received; \
            window.input_method_test_received = []; \
            received",
        )
        .expect("JS evaluation should not fail");
        let js_events = match raw_events {
            JSValue::Array(values) => values,
            _ => return Vec::new(),
        };
        js_events
            .into_iter()
            .filter_map(|js_value| {
                let JSValue::Object(mut object) = js_value else {
                    panic!("Should not put non-object into window.input_method_test_received");
                };
                let Some(JSValue::String(name)) = object.remove("name") else {
                    panic!("name should be a string");
                };
                let event = match name.as_str() {
                    "beforeinput" | "input" => {
                        let Some(JSValue::String(input_type)) = object.remove("inputType") else {
                            panic!("inputType should be a string");
                        };
                        let data = match object.remove("data").unwrap() {
                            JSValue::String(data) => Some(data),
                            JSValue::Null => None,
                            _ => panic!("data should be null or a string"),
                        };
                        if name == "beforeinput" {
                            InputMethodEvent::BeforeInput(input_type, data)
                        } else {
                            InputMethodEvent::Input(input_type, data)
                        }
                    },
                    "compositionstart" => InputMethodEvent::CompositionStart,
                    "compositionupdate" | "compositionend" => {
                        let Some(JSValue::String(data)) = object.remove("data") else {
                            panic!("data should be null or a string");
                        };
                        if name == "compositionupdate" {
                            InputMethodEvent::CompositionUpdate(data)
                        } else {
                            InputMethodEvent::CompositionEnd(data)
                        }
                    },
                    _ => panic!("Shouldn't receive any other event"),
                };
                Some(event)
            })
            .collect()
    }
}

#[test]
fn test_simple_composition() {
    let test = InputMethodTest::new(
        "<!DOCTYPE html> \
        <input id=\"my-input\" />",
    );
    test.install_event_listeners("my-input");
    test.focus("my-input");
    test.send_input_event(InputEvent::Ime(ImeEvent::Composition(CompositionEvent {
        state: CompositionState::Start,
        data: String::new(),
    })));
    test.send_input_event(InputEvent::Ime(ImeEvent::Composition(CompositionEvent {
        state: CompositionState::Update,
        data: "abc".to_owned(),
    })));
    assert_eq!(test.get_text_control_value("my-input"), "abc");
    test.send_input_event(InputEvent::Ime(ImeEvent::Composition(CompositionEvent {
        state: CompositionState::Update,
        data: "r".to_owned(),
    })));
    assert_eq!(test.get_text_control_value("my-input"), "r");
    test.send_input_event(InputEvent::Ime(ImeEvent::Composition(CompositionEvent {
        state: CompositionState::End,
        data: "r".to_owned(),
    })));
    assert_eq!(test.get_text_control_value("my-input"), "r");
    assert_eq!(
        test.drain_received_events().as_slice(),
        &[
            InputMethodEvent::CompositionStart,
            InputMethodEvent::CompositionUpdate("abc".to_owned()),
            InputMethodEvent::BeforeInput(
                "insertCompositionText".to_owned(),
                Some("abc".to_owned())
            ),
            InputMethodEvent::Input("insertCompositionText".to_owned(), Some("abc".to_owned())),
            InputMethodEvent::CompositionUpdate("r".to_owned()),
            InputMethodEvent::BeforeInput("insertCompositionText".to_owned(), Some("r".to_owned())),
            InputMethodEvent::Input("insertCompositionText".to_owned(), Some("r".to_owned())),
            InputMethodEvent::CompositionEnd("r".to_owned()),
        ]
    );
}

#[test]
fn test_attempting_to_cancel_beforeinput_composition() {
    let test = InputMethodTest::new(
        "<!DOCTYPE html> \
        <input id=\"my-input\" /> \
        <script> \
            document.getElementById('my-input').addEventListener('beforeinput', (event) => { \
                event.preventDefault(); \
            }); \
        </script>",
    );
    test.install_event_listeners("my-input");
    test.focus("my-input");
    test.send_input_event(InputEvent::Keyboard(KeyboardEvent::from_state_and_key(
        KeyState::Down,
        Key::Character("a".to_owned()),
    )));
    test.send_input_event(InputEvent::Keyboard(KeyboardEvent::from_state_and_key(
        KeyState::Up,
        Key::Character("a".to_owned()),
    )));
    assert!(test.get_text_control_value("my-input").is_empty());
    test.send_input_event(InputEvent::Ime(ImeEvent::Composition(CompositionEvent {
        state: CompositionState::Start,
        data: String::new(),
    })));
    test.send_input_event(InputEvent::Ime(ImeEvent::Composition(CompositionEvent {
        state: CompositionState::Update,
        data: "abc".to_owned(),
    })));
    test.send_input_event(InputEvent::Ime(ImeEvent::Composition(CompositionEvent {
        state: CompositionState::End,
        data: "abc".to_owned(),
    })));
    assert_eq!(test.get_text_control_value("my-input"), "abc");
    assert_eq!(
        test.drain_received_events().as_slice(),
        &[
            InputMethodEvent::BeforeInput("insertText".to_owned(), Some("a".to_owned())),
            InputMethodEvent::CompositionStart,
            InputMethodEvent::CompositionUpdate("abc".to_owned()),
            InputMethodEvent::BeforeInput(
                "insertCompositionText".to_owned(),
                Some("abc".to_owned())
            ),
            InputMethodEvent::Input("insertCompositionText".to_owned(), Some("abc".to_owned())),
            InputMethodEvent::CompositionEnd("abc".to_owned()),
        ]
    );
}
