/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Javascript Evaluator API unit tests.
mod common;

use std::rc::Rc;

use servo::{JSValue, JavaScriptEvaluationError, WebViewBuilder};

use crate::common::{ServoTest, WebViewDelegateImpl, evaluate_javascript};

#[test]
fn test_evaluate_javascript_basic() {
    let servo_test = ServoTest::new();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .build();

    let result = evaluate_javascript(&servo_test, webview.clone(), "undefined");
    assert_eq!(result, Ok(JSValue::Undefined));

    let result = evaluate_javascript(&servo_test, webview.clone(), "null");
    assert_eq!(result, Ok(JSValue::Null));

    let result = evaluate_javascript(&servo_test, webview.clone(), "42");
    assert_eq!(result, Ok(JSValue::Number(42.0)));

    let result = evaluate_javascript(&servo_test, webview.clone(), "3 + 4");
    assert_eq!(result, Ok(JSValue::Number(7.0)));

    let result = evaluate_javascript(&servo_test, webview.clone(), "'abc' + 'def'");
    assert_eq!(result, Ok(JSValue::String("abcdef".into())));

    let result = evaluate_javascript(&servo_test, webview.clone(), "let foo = {blah: 123}; foo");
    assert!(matches!(result, Ok(JSValue::Object(_))));
    if let Ok(JSValue::Object(values)) = result {
        assert_eq!(values.len(), 1);
        assert_eq!(values.get("blah"), Some(&JSValue::Number(123.0)));
    }

    let result = evaluate_javascript(&servo_test, webview.clone(), "[1, 2, 3, 4]");
    let expected = JSValue::Array(vec![
        JSValue::Number(1.0),
        JSValue::Number(2.0),
        JSValue::Number(3.0),
        JSValue::Number(4.0),
    ]);
    assert_eq!(result, Ok(expected));

    let result = evaluate_javascript(&servo_test, webview.clone(), "window");
    assert!(matches!(result, Ok(JSValue::Window(..))));

    let result = evaluate_javascript(&servo_test, webview.clone(), "document.body");
    assert!(matches!(result, Ok(JSValue::Element(..))));

    let result = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "document.body.attachShadow({mode: 'open'})",
    );
    assert!(matches!(result, Ok(JSValue::ShadowRoot(..))));

    let result = evaluate_javascript(&servo_test, webview.clone(), "document.body.shadowRoot");
    assert!(matches!(result, Ok(JSValue::ShadowRoot(..))));

    let result = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "document.body.innerHTML += '<iframe>'; frames[0]",
    );
    assert!(matches!(result, Ok(JSValue::Frame(..))));

    let result = evaluate_javascript(&servo_test, webview.clone(), "lettt badsyntax = 123");
    assert_eq!(result, Err(JavaScriptEvaluationError::CompilationFailure));

    let result = evaluate_javascript(&servo_test, webview.clone(), "throw new Error()");
    assert!(matches!(
        result,
        Err(JavaScriptEvaluationError::EvaluationFailure(_))
    ));
}

#[test]
fn test_evaluate_javascript_panic() {
    let servo_test = ServoTest::new();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .build();

    let input = "location";
    let result = evaluate_javascript(&servo_test, webview.clone(), input);
    assert!(matches!(result, Ok(JSValue::Object(..))));
}
