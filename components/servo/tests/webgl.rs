/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! WebGL object lifetime tests.
mod common;

use std::rc::Rc;

use servo::{JSValue, WebViewBuilder};
use servo_config::prefs::Preferences;
use url::Url;

use crate::common::{ServoTest, WebViewDelegateImpl, evaluate_javascript};

/// Builds many VAOs. Every other one is made the way three.js does it, with the
/// attribute and index buffers created before the VAO that points at them; the
/// rest create the VAO first. The GC finalizes objects in an order the test
/// cannot control, so mixing both makes it likelier that some buffer is
/// finalized before the VAO that still points at it. Half of the VAOs become
/// garbage straight away and are collected by `TestUtils.gc()`. The other half
/// stay reachable until the script thread shuts down and runs its final GC.
const CREATE_SCENES: &str = r#"
(() => {
  function vaoApi(gl) {
    if (gl instanceof WebGL2RenderingContext) {
      return {
        create: () => gl.createVertexArray(),
        bind: vao => gl.bindVertexArray(vao),
      };
    }
    const ext = gl.getExtension("OES_vertex_array_object");
    return ext && {
      create: () => ext.createVertexArrayOES(),
      bind: vao => ext.bindVertexArrayOES(vao),
    };
  }

  function newBuffer(gl, target, bytes) {
    const buffer = gl.createBuffer();
    gl.bindBuffer(target, buffer);
    gl.bufferData(target, new Uint8Array(bytes), gl.STATIC_DRAW);
    return buffer;
  }

  function populate(gl, api, count) {
    const meshes = [];
    for (let i = 0; i < count; i++) {
      const vaoFirst = i % 2 == 1;
      const earlyVao = vaoFirst ? api.create() : null;
      const positions = newBuffer(gl, gl.ARRAY_BUFFER, 36);
      const normals = newBuffer(gl, gl.ARRAY_BUFFER, 36);
      gl.bindBuffer(gl.ARRAY_BUFFER, null);

      const vao = earlyVao || api.create();
      api.bind(vao);
      gl.bindBuffer(gl.ARRAY_BUFFER, positions);
      gl.enableVertexAttribArray(0);
      gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 0, 0);
      gl.bindBuffer(gl.ARRAY_BUFFER, normals);
      gl.enableVertexAttribArray(1);
      gl.vertexAttribPointer(1, 3, gl.FLOAT, false, 0, 0);
      // Bound while the VAO is bound, so the VAO holds the index buffer.
      newBuffer(gl, gl.ELEMENT_ARRAY_BUFFER, 12);
      api.bind(null);
      gl.bindBuffer(gl.ARRAY_BUFFER, null);

      meshes.push(vao);
    }
    return meshes;
  }

  // Reports "ok" or the reason each context type could not be used.
  const status = {};
  window.liveScenes = [];
  for (const type of ["webgl2", "webgl"]) {
    const canvas = document.createElement("canvas");
    let creationError = "no webglcontextcreationerror event";
    canvas.addEventListener("webglcontextcreationerror", event => {
      creationError = `webglcontextcreationerror: ${event.statusMessage}`;
    });
    const gl = canvas.getContext(type);
    if (!gl) {
      status[type] = `getContext returned null (${creationError})`;
      continue;
    }
    const api = vaoApi(gl);
    if (!api) {
      status[type] = "OES_vertex_array_object is not supported";
      continue;
    }
    populate(gl, api, 64);
    window.liveScenes.push({ gl, meshes: populate(gl, api, 64) });
    status[type] = "ok";
  }
  TestUtils.gc();
  return status;
})()
"#;

/// A VAO and the buffers it points at can be finalized in the same GC, in any
/// order. This happens mid-page, and always at shutdown when the script
/// thread runs its final GC. With the default `hard_fail` option, a panic in
/// the script thread aborts this test process.
#[test]
fn test_vertex_array_objects_and_buffers_collected_together() {
    assert!(
        cfg!(feature = "webgl"),
        "Build this test with the `webgl` feature, or getContext() always returns null",
    );

    let servo_test = ServoTest::new_with_builder(|builder| {
        let mut preferences = Preferences::default();
        preferences.network_http_proxy_uri = String::new();
        preferences.network_https_proxy_uri = String::new();
        preferences.dom_webgl2_enabled = true;
        preferences.dom_testutils_enabled = true;
        builder.preferences(preferences)
    });

    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .url(Url::parse("data:text/html,<!DOCTYPE html>").unwrap())
        .delegate(delegate.clone())
        .build();

    let result = evaluate_javascript(&servo_test, webview.clone(), CREATE_SCENES);
    let Ok(JSValue::Object(status)) = &result else {
        panic!("Setting up the WebGL scenes failed: {result:?}");
    };
    eprintln!("WebGL setup: {status:?}");
    // three.js uses WebGL 2, so that is the case that must run. WebGL 1 shares
    // the same VertexArrayObject code and is only exercised when available.
    assert_eq!(
        status.get("webgl2"),
        Some(&JSValue::String("ok".into())),
        "WebGL 2 is needed to reproduce the three.js case",
    );

    // The page must still answer after the mid-page collection.
    assert_eq!(
        evaluate_javascript(&servo_test, webview.clone(), "true"),
        Ok(JSValue::Boolean(true)),
    );
    assert_eq!(
        delegate.crashes.get(),
        0,
        "The page should not have crashed"
    );

    // Same order as the reported embedder: the WebView goes first, then Servo,
    // whose shutdown runs the script thread's final GC.
    drop(webview);
    drop(servo_test);
}
