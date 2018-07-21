// These tests rely on the User Agent providing an implementation of the
// WebXR Testing API (https://github.com/immersive-web/webxr-test-api).
//
// In Chromium-based browsers, this implementation is provided by a JavaScript
// shim in order to reduce the amount of test-only code shipped to users. To
// enable these tests the browser must be run with these options:
//
//   --enable-blink-features=MojoJS,MojoJSTest

function xr_promise_test(func, name, properties) {
  promise_test(async (t) => {
    // Perform any required test setup:

    if (window.XRTest === undefined) {
      // Chrome setup
      await loadChromiumResources;
    }

    return func(t);
  }, name, properties);
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
  callback(window.XRDevice, 'XRDevice');
  callback(window.XRSession, 'XRSession');
  callback(window.XRSessionCreationOptions, 'XRSessionCreationOptions');
  callback(window.XRFrameRequestCallback, 'XRFrameRequestCallback');
  callback(window.XRPresentationContext, 'XRPresentationContext');
  callback(window.XRFrame, 'XRFrame');
  callback(window.XRView, 'XRView');
  callback(window.XRViewport, 'XRViewport');
  callback(window.XRDevicePose, 'XRDevicePose');
  callback(window.XRLayer, 'XRLayer');
  callback(window.XRWebGLLayer, 'XRWebGLLayer');
  callback(window.XRWebGLLayerInit, 'XRWebGLLayerInit');
  callback(window.XRCoordinateSystem, 'XRCoordinateSystem');
  callback(window.XRFrameOfReference, 'XRFrameOfReference');
  callback(window.XRStageBounds, 'XRStageBounds');
  callback(window.XRStageBoundsPoint, 'XRStageBoundsPoint');
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
   '/gen/ui/gfx/geometry/mojo/geometry.mojom.js',
   '/gen/mojo/public/mojom/base/time.mojom.js',
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