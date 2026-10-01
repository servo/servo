/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use http::{HeaderName, HeaderValue};
use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::{Request as HyperRequest, Response as HyperResponse};
use net::test_util::{make_body, make_server};
use servo::{JSValue, WebViewBuilder};
use servo_config::opts;

#[expect(dead_code)]
mod common;

use common::{ServoTest, WebViewDelegateImpl, evaluate_javascript};

#[test]
fn test_unminify_script() {
    let servo_test = ServoTest::new_with_builder(|builder| {
        let mut opts = opts::Opts::default();
        opts.unminify_js = true;
        builder.opts(opts)
    });

    static PAGE: &str = "
loaded = false;
<script src=\"test.js\"></script>
<script>
  success2 = false;
  onload = () => { loaded = true; }
</script>
<script src=\"test2.js\" type=module></script>
";
    static SCRIPT: &[u8] = b"success = true;";
    static SCRIPT_MODULE: &[u8] = b"success2 = true;";
    let handler =
        move |request: HyperRequest<Incoming>,
              response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
            if request.uri().path().contains("test.js") {
                response.headers_mut().insert(
                    HeaderName::from_static("content-type"),
                    HeaderValue::from_static("text/javascript"),
                );
                *response.body_mut() = make_body(SCRIPT.to_vec());
            } else if request.uri().path().contains("test2.js") {
                response.headers_mut().insert(
                    HeaderName::from_static("content-type"),
                    HeaderValue::from_static("text/javascript"),
                );
                *response.body_mut() = make_body(SCRIPT_MODULE.to_vec());
            } else {
                response.headers_mut().insert(
                    HeaderName::from_static("content-type"),
                    HeaderValue::from_static("text/html"),
                );
                *response.body_mut() = make_body(PAGE.as_bytes().to_vec());
            }
        };
    let (server, url) = make_server(handler);

    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(url.url().into_url())
        .build();
    servo_test.spin(|| !delegate.load_status_changed.get());

    let loaded = evaluate_javascript(&servo_test, webview.clone(), "loaded");
    assert_eq!(loaded, Ok(JSValue::Boolean(true)));

    let classic = evaluate_javascript(&servo_test, webview.clone(), "success");
    assert_eq!(classic, Ok(JSValue::Boolean(true)));

    let module = evaluate_javascript(&servo_test, webview.clone(), "success2");
    assert_eq!(module, Ok(JSValue::Boolean(true)));

    server.close();
}
