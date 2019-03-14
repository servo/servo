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
    name, func, fakeDeviceInit, sessionOptions, properties) {
  let testDeviceController;
  let testSession;

  const webglCanvas = document.getElementsByTagName('canvas')[0];
  if (!webglCanvas) {
    promise_test(async (t) => {
      Promise.reject('xr_session_promise_test requires a canvas on the page!');
    }, name, properties);
  }
  let gl = webglCanvas.getContext('webgl', {alpha: false, antialias: false});

  xr_promise_test(
      name,
      (t) =>
          XRTest.simulateDeviceConnection(fakeDeviceInit)
              .then((controller) => {
                testDeviceController = controller;
                return gl.makeXRCompatible();
              })
              .then(() => new Promise((resolve, reject) => {
                      // Perform the session request in a user gesture.
                      XRTest.simulateUserActivation(() => {
                        navigator.xr.requestSession(sessionOptions)
                            .then((session) => {
                              testSession = session;
                              // Session must have a baseLayer or frame requests
                              // will be ignored.
                              session.updateRenderState({
                                  baseLayer: new XRWebGLLayer(session, gl),
                                  outputContext: getOutputContext()
                              });
                              resolve(func(session, testDeviceController, t));
                            })
                            .catch((err) => {
                              reject(
                                  'Session with params ' +
                                  JSON.stringify(sessionOptions) +
                                  ' was rejected on device ' +
                                  JSON.stringify(fakeDeviceInit) +
                                  ' with error: ' + err);
                            });
                      });
                    }))
              .then(() => {
                // Cleanup system state.
                testSession.end().catch(() => {});
                XRTest.simulateDeviceDisconnection();
              }),
      properties);
}

// A utility function to create an output context as required by non-immersive
// sessions.
// https://immersive-web.github.io/webxr/#xrsessioncreationoptions-interface
function getOutputContext() {
  let outputCanvas = document.createElement('canvas');
  document.body.appendChild(outputCanvas);
  return outputCanvas.getContext('xrpresent');
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
  callback(window.XRLayer, 'XRLayer');
  callback(window.XRWebGLLayer, 'XRWebGLLayer');
  callback(window.XRWebGLLayerInit, 'XRWebGLLayerInit');
  callback(window.XRCoordinateSystem, 'XRCoordinateSystem');
  callback(window.XRFrameOfReference, 'XRFrameOfReference');
  callback(window.XRStageBounds, 'XRStageBounds');
  callback(window.XRSessionEvent, 'XRSessionEvent');
  callback(window.XRCoordinateSystemEvent, 'XRCoordinateSystemEvent');
}

// Code for loading test api in chromium.
let loadChromiumResources = Promise.resolve().then(() => {
  if (!MojoInterfaceInterceptor) {
    // Do nothing on non-Chromium-based browsers or when the Mojo bindings are
    // not present in the global namespace.
    return;
  }

  let chain = Promise.resolve();
  ['/gen/layout_test_data/mojo/public/js/mojo_bindings.js',
   '/gen/mojo/public/mojom/base/time.mojom.js',
   '/gen/gpu/ipc/common/mailbox_holder.mojom.js',
   '/gen/gpu/ipc/common/sync_token.mojom.js',
   '/gen/ui/display/mojo/display.mojom.js',
   '/gen/ui/gfx/geometry/mojo/geometry.mojom.js',
   '/gen/ui/gfx/mojo/gpu_fence_handle.mojom.js',
   '/gen/ui/gfx/mojo/transform.mojom.js',
   '/gen/device/vr/public/mojom/vr_service.mojom.js',
   '/resources/chromium/webxr-test.js', '/resources/testdriver.js',
   '/resources/testdriver-vendor.js',
  ].forEach(path => {
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