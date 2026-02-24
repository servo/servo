/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! [`UserContentManager`] API unit tests.
mod common;

use std::cell::RefCell;
use std::rc::Rc;

use net::test_util::{make_body, make_server};
use servo::user_contents::UserStyleSheet;
use servo::{
    CreateNewWebViewRequest, JSValue, LoadStatus, RenderingContext, Servo, UserContentManager,
    UserScript, WebView, WebViewBuilder, WebViewDelegate,
};
use url::Url;

use crate::common::{ServoTest, evaluate_javascript};

#[test]
fn test_user_content_manager_empty() {
    let servo_test = ServoTest::new();
    let user_content_manager = UserContentManager::new(servo_test.servo());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .user_content_manager(Rc::new(user_content_manager))
        .url(Url::parse("data:text/html,Hello World").unwrap())
        .build();

    let load_webview = webview.clone();
    let _ = servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);
    let result = evaluate_javascript(&servo_test, webview.clone(), "window.fromUserContentScript");
    assert_eq!(result, Ok(JSValue::Undefined));
}

#[test]
fn test_user_content_manager_user_script() {
    let servo_test = ServoTest::new();

    // Use a http server instead of a data url to allow the `webview.reload()` call below to reuse
    // the exisitng script thread. This is necessary to test that mutations on a `UserContentManager`
    // take effect on script threads created before the mutation.
    let (_, url) = make_server(move |_, response| {
        *response.body_mut() = make_body(b"<!DOCTYPE html>\nHello".to_vec());
    });

    let user_content_manager = Rc::new(UserContentManager::new(servo_test.servo()));
    user_content_manager.add_script(Rc::new("window.fromUserContentScript = 42;".into()));

    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .user_content_manager(user_content_manager.clone())
        .url(url.into_url())
        .build();

    let load_webview = webview.clone();
    let _ = servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);
    let result = evaluate_javascript(&servo_test, webview.clone(), "window.fromUserContentScript");
    assert_eq!(result, Ok(JSValue::Number(42.0)));

    // Add a second user script to the `UserContentManager`.
    let second_user_script = Rc::new(UserScript::from("window.fromSecondUserContentScript = 32;"));
    user_content_manager.add_script(second_user_script.clone());

    // The second user script must immediately take effect in any new WebViews.
    let new_webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .user_content_manager(user_content_manager.clone())
        .url(Url::parse("data:text/html,<!DOCTYPE html>").unwrap())
        .build();
    let load_webview = new_webview.clone();
    let _ = servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);
    let result = evaluate_javascript(
        &servo_test,
        new_webview,
        "window.fromSecondUserContentScript",
    );
    assert_eq!(result, Ok(JSValue::Number(32.0)));

    // The existing page in the first webview must not be affected since we haven't reloaded yet.
    let result = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "window.fromSecondUserContentScript",
    );
    assert_eq!(result, Ok(JSValue::Undefined));

    // Now trigger a reload and ensure the second user script has effect on the page.
    webview.reload();

    let load_webview = webview.clone();
    let _ = servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);
    let result = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "window.fromSecondUserContentScript",
    );

    assert_eq!(result, Ok(JSValue::Number(32.0)));

    // Test that removing the user script works. Trigger a reload and ensure the second user script
    // no longer has effect on the page.
    user_content_manager.remove_script(second_user_script);
    webview.reload();

    let load_webview = webview.clone();
    let _ = servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);
    let result = evaluate_javascript(&servo_test, webview, "window.fromSecondUserContentScript");

    assert_eq!(result, Ok(JSValue::Undefined));
}

#[test]
fn test_user_content_manager_for_auxiliary_webviews() {
    let servo_test = ServoTest::new();
    struct WebViewAuxiliaryTestDelegate {
        servo: Servo,
        rendering_context: Rc<dyn RenderingContext>,
        auxiliary_webview: RefCell<Option<WebView>>,
    }

    impl WebViewDelegate for WebViewAuxiliaryTestDelegate {
        fn request_create_new(&self, _parent_webview: WebView, request: CreateNewWebViewRequest) {
            let user_content_manager_for_auxiliary_webview = UserContentManager::new(&self.servo);
            // Add a different user script to the `UserContentManager` of auxiliary webview.
            user_content_manager_for_auxiliary_webview.add_script(Rc::new(
                "window.fromAuxiliaryUserContentScript = 32;".into(),
            ));
            let auxiliary_webview = request
                .builder(self.rendering_context.clone())
                .user_content_manager(Rc::new(user_content_manager_for_auxiliary_webview))
                .build();
            self.auxiliary_webview
                .borrow_mut()
                .replace(auxiliary_webview.clone());
        }
    }

    let delegate = Rc::new(WebViewAuxiliaryTestDelegate {
        servo: servo_test.servo.clone(),
        rendering_context: servo_test.rendering_context.clone(),
        auxiliary_webview: RefCell::new(None),
    });

    let user_content_manager = UserContentManager::new(servo_test.servo());
    user_content_manager.add_script(Rc::new("window.fromUserContentScript = 42;".into()));

    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .user_content_manager(Rc::new(user_content_manager))
        .url(
            Url::parse(
                "data:text/html,<!DOCTYPE html>\
                <script>\
                    onload = () => window.open('data:text/html,<title>Auxiliary WebView</title>')\
                </script>",
            )
            .unwrap(),
        )
        .build();

    let load_webview = webview.clone();
    let delegate_clone = delegate.clone();
    let _ = servo_test.spin(move || {
        load_webview.load_status() != LoadStatus::Complete ||
            delegate_clone
                .auxiliary_webview
                .borrow()
                .as_ref()
                .is_none_or(|auxiliary_webview| {
                    auxiliary_webview.page_title() != Some("Auxiliary WebView".into())
                })
    });

    let result = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "[ window.fromUserContentScript, window.fromAuxiliaryUserContentScript ]",
    );
    assert_eq!(
        result,
        Ok(JSValue::Array(vec![
            JSValue::Number(42.0),
            JSValue::Undefined
        ]))
    );

    let auxiliary_webview = delegate
        .auxiliary_webview
        .borrow_mut()
        .take()
        .expect("Gauranteed by spin");

    let result = evaluate_javascript(
        &servo_test,
        auxiliary_webview.clone(),
        "[ window.fromUserContentScript, window.fromAuxiliaryUserContentScript ]",
    );

    assert_eq!(
        result,
        Ok(JSValue::Array(vec![
            JSValue::Undefined,
            JSValue::Number(32.0),
        ]))
    );
}

#[test]
fn test_user_content_manager_for_user_stylesheets() {
    let servo_test = ServoTest::new();

    let user_content_manager = Rc::new(UserContentManager::new(servo_test.servo()));

    #[cfg(not(target_os = "windows"))]
    let url = Url::from_file_path("/test/test.css").unwrap();
    #[cfg(target_os = "windows")]
    let url = Url::from_file_path("C:\\test\\test.css").unwrap();

    let user_stylesheet = Rc::new(UserStyleSheet::new(
        "div { width: 100px; height: 50px }\
        p { width: 200px; height: 200px }"
            .into(),
        url,
    ));
    user_content_manager.add_stylesheet(user_stylesheet.clone());

    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .user_content_manager(user_content_manager.clone())
        .url(
            Url::parse(
                "data:text/html,<!DOCTYPE html>\
                        <style>p { width: 300px; height: 300px }</style>\
                        <div id='div1'></div><p id='p1'>test paragraph</p>",
            )
            .unwrap(),
        )
        .build();

    let load_webview = webview.clone();
    let _ = servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);

    let result = evaluate_javascript(
        &servo_test,
        webview.clone(),
        "[ div1.offsetWidth, div1.offsetHeight, p1.offsetWidth, p1.offsetHeight ]",
    );
    assert_eq!(
        result,
        Ok(JSValue::Array(vec![
            // `div` elements uses the rules from the user stylesheet since the author stylesheet doesn't
            // have any rules that match `div`s.
            JSValue::Number(100.0),
            JSValue::Number(50.0),
            // `p` element uses the rules from author stylesheet as they have precendece over user
            // rules from user stylesheets.
            JSValue::Number(300.0),
            JSValue::Number(300.0),
        ]))
    );

    // Test that removing the stylesheet works.
    user_content_manager.remove_stylesheet(user_stylesheet);
    webview.reload();

    let load_webview = webview.clone();
    let _ = servo_test.spin(move || load_webview.load_status() != LoadStatus::Complete);

    let result = evaluate_javascript(&servo_test, webview.clone(), "div1.offsetHeight");

    assert_eq!(result, Ok(JSValue::Number(0.0)));
}
