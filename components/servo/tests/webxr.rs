/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg(all(feature = "webxr", feature = "webgl"))]

mod common;

use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::header::{self, HeaderValue};
use hyper::{Request as HyperRequest, Response as HyperResponse};
use net::test_util::{make_body, make_server};
use servo::webxr::{HeadlessMockDiscovery, MainThreadRegistry, WebXrRegistry};
#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
use servo::webxr::{OpenXrAppInfo, OpenXrDiscovery};
use servo::{JSValue, Preferences, WebViewBuilder};

use crate::common::{ServoTest, WebViewDelegateImpl, evaluate_javascript};

/// Registers both `HeadlessMockDiscovery` and (on Linux) `OpenXrDiscovery`,
/// mirroring `servoshell`'s `XrDiscoveryWebXrRegistry`.
struct TestWebXrRegistry {
    mock_enabled: Arc<AtomicBool>,
}

impl WebXrRegistry for TestWebXrRegistry {
    fn register(&self, xr: &mut MainThreadRegistry) {
        xr.register_mock(HeadlessMockDiscovery::new(self.mock_enabled.clone()));
        #[cfg(all(target_os = "linux", not(target_env = "ohos")))]
        {
            let app_info = OpenXrAppInfo::new("ServoTest", 0, "Servo", 0);
            xr.register(OpenXrDiscovery::new(None, app_info));
        }
    }
}

fn webxr_test_page_handler(
    _request: HyperRequest<Incoming>,
    response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>,
) {
    static HTML: &[u8] = br#"<!doctype html>
<html><head><meta charset="utf-8"><title>Servo WebXR Integration Test</title></head>
<body>
<canvas id="c1" width="128" height="128"></canvas>
<canvas id="c2" width="128" height="128"></canvas>
<script>
window.__runWebXrTest = async function() {
  const out = {};
  out.hasXr = !!navigator.xr;
  out.hasTest = !!(navigator.xr && navigator.xr.test);
  // With no active OpenXR runtime and no simulated device connected, this must resolve false.
  out.supportedBefore = await navigator.xr.isSessionSupported('immersive-vr');

  const identity = { position: [0, 0, 0], orientation: [0, 0, 0, 1] };
  const proj = [
    1, 0, 0, 0,
    0, 1, 0, 0,
    0, 0, -1, -1,
    0, 0, -0.2, 0,
  ];
  const controller = await navigator.xr.test.simulateDeviceConnection({
    supportsImmersive: true,
    supportedModes: ['immersive-vr'],
    supportedFeatures: ['viewer', 'local', 'local-floor'],
    views: [
      {
        eye: 'left',
        projectionMatrix: proj,
        viewOffset: { position: [-0.032, 0, 0], orientation: [0, 0, 0, 1] },
        resolution: { width: 128, height: 128 },
      },
      {
        eye: 'right',
        projectionMatrix: proj,
        viewOffset: { position: [0.032, 0, 0], orientation: [0, 0, 0, 1] },
        resolution: { width: 128, height: 128 },
      },
    ],
    viewerOrigin: identity,
    floorOrigin: { position: [0, -1.6, 0], orientation: [0, 0, 0, 1] },
  });

  out.supportedAfter = await navigator.xr.isSessionSupported('immersive-vr');

  async function runSession(canvasId, glKind, sessionFirst) {
    const canvas = document.getElementById(canvasId);
    let gl, session;
    if (sessionFirst) {
      session = await navigator.xr.requestSession('immersive-vr');
      gl = canvas.getContext(glKind, { xrCompatible: true });
    } else {
      gl = canvas.getContext(glKind, { xrCompatible: true });
      await gl.makeXRCompatible();
      session = await navigator.xr.requestSession('immersive-vr');
    }
    const refSpace = await session.requestReferenceSpace('local');
    const layer = new XRWebGLLayer(session, gl);
    session.updateRenderState({ baseLayer: layer });

    let ended = false;
    session.addEventListener('end', () => { ended = true; });

    const res = await new Promise((resolve, reject) => {
      let frames = 0;
      let viewsSeen = 0;
      let glError = 0;
      const onFrame = async (t, frame) => {
        try {
          frames++;
          const pose = frame.getViewerPose(refSpace);
          if (pose) {
            viewsSeen = pose.views.length;
            gl.bindFramebuffer(gl.FRAMEBUFFER, layer.framebuffer);
            for (const view of pose.views) {
              const vp = layer.getViewport(view);
              gl.viewport(vp.x, vp.y, vp.width, vp.height);
              gl.clearColor(1, 0, 0, 1);
              gl.clear(gl.COLOR_BUFFER_BIT);
            }
            const err = gl.getError();
            if (err !== 0 && glError === 0) glError = err;
          }
          if (frames >= 3) {
            await session.end();
            setTimeout(() => resolve({
              frames,
              viewsSeen,
              glError,
              ended,
              fbWidth: layer.framebufferWidth,
              fbHeight: layer.framebufferHeight,
            }), 50);
            return;
          }
          session.requestAnimationFrame(onFrame);
        } catch (e) {
          reject(e);
        }
      };
      session.requestAnimationFrame(onFrame);
    });
    return res;
  }

  out.webgl1 = await runSession('c1', 'webgl', false);
  out.webgl2 = await runSession('c2', 'webgl2', true);
  await controller.disconnect();
  out.supportedAfterDisconnect = await navigator.xr.isSessionSupported('immersive-vr');
  window.__testResult = JSON.stringify(out);
};
</script>
</body></html>
"#;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    *response.body_mut() = make_body(HTML.to_vec());
}

fn wait_for_js_string(
    servo_test: &ServoTest,
    webview: &servo::WebView,
    expr: &str,
    timeout: Duration,
) -> String {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Ok(JSValue::String(value)) = evaluate_javascript(servo_test, webview.clone(), expr) {
            return value;
        }
        servo_test.servo().spin_event_loop();
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("Timed out waiting for JS expression {expr}");
}

#[test]
fn test_webxr_linux_openxr_and_mock_sessions_end_to_end() {
    #[cfg(all(target_os = "linux", not(target_env = "ohos")))]
    let prev_runtime = {
        let prev = std::env::var_os("XR_RUNTIME_JSON");
        // SAFETY: Single test function in this integration test binary.
        unsafe {
            std::env::set_var("XR_RUNTIME_JSON", "/nonexistent/servo-openxr-test.json");
        }
        prev
    };

    let servo_test = ServoTest::new_with_builder(|builder| {
        let mut prefs = Preferences::default();
        prefs.network_http_proxy_uri = String::new();
        prefs.network_https_proxy_uri = String::new();
        prefs.dom_webxr_enabled = true;
        prefs.dom_webxr_openxr_enabled = true;
        prefs.dom_webxr_test = true;
        prefs.dom_webxr_unsafe_assume_user_intent = true;
        prefs.dom_webgl2_enabled = true;
        builder.preferences(prefs)
    });
    servo_test
        .servo()
        .register_webxr_registry(Box::new(TestWebXrRegistry {
            mock_enabled: Arc::new(AtomicBool::new(true)),
        }));

    let (server, url) = make_server(webxr_test_page_handler);
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .url(url.as_url().clone())
        .build();
    let delegate_clone = delegate.clone();
    servo_test.spin(move || !delegate_clone.load_status_changed.get());

    let _ = evaluate_javascript(&servo_test, webview.clone(), "window.__runWebXrTest();");
    let json = wait_for_js_string(
        &servo_test,
        &webview,
        "window.__testResult",
        Duration::from_secs(15),
    );
    let _ = server.close();
    drop(webview);
    drop(servo_test);

    #[cfg(all(target_os = "linux", not(target_env = "ohos")))]
    // SAFETY: Single test function in this integration test binary.
    unsafe {
        if let Some(prev) = prev_runtime {
            std::env::set_var("XR_RUNTIME_JSON", prev);
        } else {
            std::env::remove_var("XR_RUNTIME_JSON");
        }
    }

    assert!(json.contains("\"hasXr\":true"), "unexpected result: {json}");
    assert!(
        json.contains("\"hasTest\":true"),
        "unexpected result: {json}"
    );
    assert!(
        json.contains("\"supportedBefore\":false"),
        "unexpected result: {json}"
    );
    assert!(
        json.contains("\"supportedAfter\":true"),
        "unexpected result: {json}"
    );
    assert!(
        json.contains("\"supportedAfterDisconnect\":false"),
        "unexpected result: {json}"
    );
    assert!(
        json.contains("\"viewsSeen\":2"),
        "expected stereo views in result: {json}"
    );
    assert!(
        json.contains("\"glError\":0"),
        "expected zero GL errors in result: {json}"
    );
    assert!(
        json.contains("\"ended\":true"),
        "expected session end event in result: {json}"
    );
    assert!(
        json.contains("\"webgl1\":") && json.contains("\"webgl2\":"),
        "expected both webgl1 and webgl2 results: {json}"
    );
}
