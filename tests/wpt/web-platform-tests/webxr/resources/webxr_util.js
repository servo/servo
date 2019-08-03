// These tests rely on the User Agent providing an implementation of the
// WebXR Testing API (https://github.com/immersive-web/webxr-test-api).
//
// In Chromium-based browsers, this implementation is provided by a JavaScript
// shim in order to reduce the amount of test-only code shipped to users. To
// enable these tests the browser must be run with these options:
//
//   --enable-blink-features=MojoJS,MojoJSTest

function xr_promise_test(name, func, properties) {
  promise_test(async (t) => {
    // Perform any required test setup:

    if (window.XRTest === undefined) {
      // Chrome setup
      await loadChromiumResources;
    }

    return func(t);
  }, name, properties);
}

// A test function which runs through the common steps of requesting a session.
// Calls the passed in test function with the session, the controller for the
// device, and the test object.
// Requires a webglCanvas on the page.
function xr_session_promise_test(
    name, func, fakeDeviceInit, sessionMode, sessionInit, properties) {
  let testDeviceController;
  let testSession;
  let sessionObjects = {};

  const webglCanvas = document.getElementsByTagName('canvas')[0];
  // We can't use assert_true here because it causes the wpt testharness to treat
  // this as a test page and not as a test.
  if (!webglCanvas) {
    promise_test(async (t) => {
      Promise.reject('xr_session_promise_test requires a canvas on the page!');
    }, name, properties);
  }
  let gl = webglCanvas.getContext('webgl', {alpha: false, antialias: false});
  sessionObjects.gl = gl;

  xr_promise_test(
      name,
      (t) => {
          // Ensure that any pending sessions are ended and devices are
          // disconnected when done. This needs to use a cleanup function to
          // ensure proper sequencing. If this were done in a .then() for the
          // success case, a test that expected failure would already be marked
          // done at the time that runs, and the shutdown would interfere with
          // the next test which may have started already.
          t.add_cleanup(async () => {
                // If a session was created, end it.
                if (testSession) {
                  await testSession.end().catch(() => {});
                }
                // Cleanup system state.
                await navigator.xr.test.disconnectAllDevices();
          });

          return navigator.xr.test.simulateDeviceConnection(fakeDeviceInit)
              .then((controller) => {
                testDeviceController = controller;
                return gl.makeXRCompatible();
              })
              .then(() => new Promise((resolve, reject) => {
                      // Perform the session request in a user gesture.
                      navigator.xr.test.simulateUserActivation(() => {
                        navigator.xr.requestSession(sessionMode, sessionInit || {})
                            .then((session) => {
                              testSession = session;
                              session.mode = sessionMode;
                              let glLayer = new XRWebGLLayer(session, gl);
                              glLayer.context = gl;
                              // Session must have a baseLayer or frame requests
                              // will be ignored.
                              session.updateRenderState({
                                  baseLayer: glLayer
                              });
                              sessionObjects.glLayer = glLayer;
                              resolve(func(session, testDeviceController, t, sessionObjects));
                            })
                            .catch((err) => {
                              reject(
                                  'Session with params ' +
                                  JSON.stringify(sessionMode) +
                                  ' was rejected on device ' +
                                  JSON.stringify(fakeDeviceInit) +
                                  ' with error: ' + err);
                            });
                      });
              }));
      },
      properties);
}

// This functions calls a callback with each API object as specified
// by https://immersive-web.github.io/webxr/spec/latest/, allowing
// checks to be made on all ojects.
// Arguements:
//      callback: A callback function with two arguements, the first
//                being the API object, the second being the name of
//                that API object.
function forEachWebxrObject(callback) {
  callback(window.navigator.xr, 'navigator.xr');
  callback(window.XRSession, 'XRSession');
  callback(window.XRSessionCreationOptions, 'XRSessionCreationOptions');
  callback(window.XRFrameRequestCallback, 'XRFrameRequestCallback');
  callback(window.XRPresentationContext, 'XRPresentationContext');
  callback(window.XRFrame, 'XRFrame');
  callback(window.XRView, 'XRView');
  callback(window.XRViewport, 'XRViewport');
  callback(window.XRViewerPose, 'XRViewerPose');
  callback(window.XRWebGLLayer, 'XRWebGLLayer');
  callback(window.XRWebGLLayerInit, 'XRWebGLLayerInit');
  callback(window.XRCoordinateSystem, 'XRCoordinateSystem');
  callback(window.XRFrameOfReference, 'XRFrameOfReference');
  callback(window.XRStageBounds, 'XRStageBounds');
  callback(window.XRSessionEvent, 'XRSessionEvent');
  callback(window.XRCoordinateSystemEvent, 'XRCoordinateSystemEvent');
}

// Code for loading test API in Chromium.
let loadChromiumResources = Promise.resolve().then(() => {
  if (!('MojoInterfaceInterceptor' in self)) {
    // Do nothing on non-Chromium-based browsers or when the Mojo bindings are
    // not present in the global namespace.
    return;
  }

  let chromiumResources = [
    '/gen/layout_test_data/mojo/public/js/mojo_bindings.js',
    '/gen/mojo/public/mojom/base/time.mojom.js',
    '/gen/gpu/ipc/common/mailbox_holder.mojom.js',
    '/gen/gpu/ipc/common/sync_token.mojom.js',
    '/gen/ui/display/mojom/display.mojom.js',
    '/gen/ui/gfx/geometry/mojom/geometry.mojom.js',
    '/gen/ui/gfx/mojom/gpu_fence_handle.mojom.js',
    '/gen/ui/gfx/mojom/transform.mojom.js',
    '/gen/device/vr/public/mojom/vr_service.mojom.js',
    '/resources/chromium/webxr-test.js',
    '/resources/testdriver.js',
    '/resources/testdriver-vendor.js',
  ];

  // This infrastructure is also used by Chromium-specific internal tests that
  // may need additional resources (e.g. internal API extensions), this allows
  // those tests to rely on this infrastructure while ensuring that no tests
  // make it into public WPTs that rely on APIs outside of the webxr test API.
  if (typeof(additionalChromiumResources) !== 'undefined') {
    chromiumResources = chromiumResources.concat(additionalChromiumResources);
  }

  let chain = Promise.resolve();
    chromiumResources.forEach(path => {
      let script = document.createElement('script');
      script.src = path;
      script.async = false;
      chain = chain.then(() => new Promise(resolve => {
                           script.onload = () => resolve();
                         }));
      document.head.appendChild(script);
  });

  return chain;
});
