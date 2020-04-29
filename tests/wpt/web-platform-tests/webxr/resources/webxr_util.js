// These tests rely on the User Agent providing an implementation of the
// WebXR Testing API (https://github.com/immersive-web/webxr-test-api).
//
// In Chromium-based browsers, this implementation is provided by a JavaScript
// shim in order to reduce the amount of test-only code shipped to users. To
// enable these tests the browser must be run with these options:
//
//   --enable-blink-features=MojoJS,MojoJSTest

// Debugging message helper, by default does nothing. Implementations can
// override this.
var xr_debug = function(name, msg) {}
var isChromiumBased = 'MojoInterfaceInterceptor' in self;
var isWebKitBased = 'internals' in self && 'xrTest' in internals;

function xr_promise_test(name, func, properties) {
  promise_test(async (t) => {
    // Perform any required test setup:
    xr_debug(name, 'setup');

    if (isChromiumBased) {
      // Chrome setup
      await loadChromiumResources;
      xr_debug = navigator.xr.test.Debug;
    }

    if (isWebKitBased) {
      // WebKit setup
      await setupWebKitWebXRTestAPI;
    }

    // Ensure that any devices are disconnected when done. If this were done in
    // a .then() for the success case, a test that expected failure would
    // already be marked done at the time that runs and the shutdown would
    // interfere with the next test.
    t.add_cleanup(async () => {
      // Ensure system state is cleaned up.
      xr_debug(name, 'cleanup');
      await navigator.xr.test.disconnectAllDevices();
    });

    xr_debug(name, 'main');
    return func(t);
  }, name, properties);
}

// A test function which runs through the common steps of requesting a session.
// Calls the passed in test function with the session, the controller for the
// device, and the test object.
// Requires a webglCanvas on the page.
function xr_session_promise_test(
    name, func, fakeDeviceInit, sessionMode, sessionInit, properties, glcontextPropertiesParam, gllayerPropertiesParam) {
  let testDeviceController;
  let testSession;
  let sessionObjects = {};
  const glcontextProperties = (glcontextPropertiesParam) ? glcontextPropertiesParam : {};
  const gllayerProperties = (gllayerPropertiesParam) ? gllayerPropertiesParam : {};

  const webglCanvas = document.getElementsByTagName('canvas')[0];
  // We can't use assert_true here because it causes the wpt testharness to treat
  // this as a test page and not as a test.
  if (!webglCanvas) {
    promise_test(async (t) => {
      Promise.reject('xr_session_promise_test requires a canvas on the page!');
    }, name, properties);
  }
  let gl = webglCanvas.getContext('webgl', {alpha: false, antialias: false, ...glcontextProperties});
  sessionObjects.gl = gl;

  xr_promise_test(
      name,
      (t) => {
          // Ensure that any pending sessions are ended when done. This needs to
          // use a cleanup function to ensure proper sequencing. If this were
          // done in a .then() for the success case, a test that expected
          // failure would already be marked done at the time that runs, and the
          // shutdown would interfere with the next test which may have started.
          t.add_cleanup(async () => {
            // If a session was created, end it.
            if (testSession) {
              await testSession.end().catch(() => {});
            }
          });

          return navigator.xr.test.simulateDeviceConnection(fakeDeviceInit)
              .then((controller) => {
                testDeviceController = controller;
                return gl.makeXRCompatible();
              })
              .then(() => new Promise((resolve, reject) => {
                      // Perform the session request in a user gesture.
                      xr_debug(name, 'simulateUserActivation');
                      navigator.xr.test.simulateUserActivation(() => {
                        xr_debug(name, 'document.hasFocus()=' + document.hasFocus());
                        navigator.xr.requestSession(sessionMode, sessionInit || {})
                            .then((session) => {
                              xr_debug(name, 'session start');
                              testSession = session;
                              session.mode = sessionMode;
                              let glLayer = new XRWebGLLayer(session, gl, gllayerProperties);
                              glLayer.context = gl;
                              // Session must have a baseLayer or frame requests
                              // will be ignored.
                              session.updateRenderState({
                                  baseLayer: glLayer
                              });
                              sessionObjects.glLayer = glLayer;
                              xr_debug(name, 'session.visibilityState=' + session.visibilityState);
                              resolve(func(session, testDeviceController, t, sessionObjects));
                            })
                            .catch((err) => {
                              xr_debug(name, 'error: ' + err);
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


// This function wraps the provided function in a
// simulateUserActivation() call, and resolves the promise with the
// result of func(), or an error if one is thrown
function promise_simulate_user_activation(func) {
  return new Promise((resolve, reject) => {
    navigator.xr.test.simulateUserActivation(() => {
      try { let a = func(); resolve(a); } catch(e) { reject(e); }
    });
  });
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
  callback(window.XRLayer, 'XRLayer');
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
  if (!isChromiumBased) {
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
    '/resources/chromium/webxr-test-math-helper.js',
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

let setupWebKitWebXRTestAPI = Promise.resolve().then(() => {
  if (!isWebKitBased) {
    // Do nothing on non-WebKit-based browsers.
    return;
  }

  // WebKit setup. The internals object is used by the WebKit test runner
  // to provide JS access to internal APIs. In this case it's used to
  // ensure that XRTest is only exposed to wpt tests.
  navigator.xr.test = internals.xrTest;
  return Promise.resolve();
});
