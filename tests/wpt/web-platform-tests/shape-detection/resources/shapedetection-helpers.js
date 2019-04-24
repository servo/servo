'use strict';

// These tests rely on the User Agent providing an implementation of
// platform shape detection backends.
//
// In Chromium-based browsers this implementation is provided by a polyfill
// in order to reduce the amount of test-only code shipped to users. To enable
// these tests the browser must be run with these options:
//
//   --enable-blink-features=MojoJS,MojoJSTest

let loadChromiumResources = Promise.resolve().then(() => {
  if (!MojoInterfaceInterceptor) {
    // Do nothing on non-Chromium-based browsers or when the Mojo bindings are
    // not present in the global namespace.
    return;
  }

  const prefix = '/gen/services/shape_detection/public/mojom';
  let chain = Promise.resolve();
  [
    '/gen/layout_test_data/mojo/public/js/mojo_bindings.js',
    '/gen/mojo/public/mojom/base/big_buffer.mojom.js',
    '/gen/skia/public/interfaces/image_info.mojom.js',
    '/gen/skia/public/interfaces/bitmap.mojom.js',
    '/gen/ui/gfx/geometry/mojo/geometry.mojom.js',
    `${prefix}/barcodedetection.mojom.js`,
    `${prefix}/barcodedetection_provider.mojom.js`,
    `${prefix}/facedetection.mojom.js`,
    `${prefix}/facedetection_provider.mojom.js`,
    '/resources/chromium/mock-barcodedetection.js',
    '/resources/chromium/mock-facedetection.js',
  ].forEach(path => {
    // Use importScripts for workers.
    if (typeof document === 'undefined') {
      chain = chain.then(() => importScripts(path));
      return;
    }
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

/**
 * @param {String} detectionTestName
 * name of mock shape detection test interface,
 * must be the item of ["FaceDetectionTest", "BarcodeDetectionTest"]
*/
async function initialize_detection_tests(detectionTestName) {
  let detectionTest;
  // Use 'self' for workers.
  if (typeof document === 'undefined') {
    if (typeof self[detectionTestName] === 'undefined') {
      await loadChromiumResources;
    }
    detectionTest = new self[detectionTestName]();
  } else {
    if (typeof window[detectionTestName] === 'undefined') {
      await loadChromiumResources;
    }
    detectionTest = new window[detectionTestName]();
  }
  await detectionTest.initialize();
  return detectionTest;
}

function detection_test(detectionTestName, func, name, properties) {
  promise_test(async t => {
    let detectionTest = await initialize_detection_tests(detectionTestName);
    try {
      await func(t, detectionTest);
    } finally {
      await detectionTest.reset();
    };
  }, name, properties);
}

function getArrayBufferFromBigBuffer(bigBuffer) {
  if (bigBuffer.$tag == mojoBase.mojom.BigBuffer.Tags.bytes) {
    return new Uint8Array(bigBuffer.bytes).buffer;
  }
  return bigBuffer.sharedMemory.bufferHandle.mapBuffer(0,
      bigBuffer.sharedMemory.size).buffer;
}
