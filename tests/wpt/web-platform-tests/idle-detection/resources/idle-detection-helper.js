'use strict';

// These tests rely on the User Agent providing an implementation of
// platform nfc backends.
//
// In Chromium-based browsers this implementation is provided by a polyfill
// in order to reduce the amount of test-only code shipped to users. To enable
// these tests the browser must be run with these options:
//
//   --enable-blink-features=MojoJS,MojoJSTest

async function loadChromiumResources() {
  const chromiumResources = [
    '/gen/mojo/public/mojom/base/string16.mojom.js',
    '/gen/mojo/public/mojom/base/time.mojom.js',
    '/gen/third_party/blink/public/mojom/idle/idle_manager.mojom.js'
  ];

  await loadMojoResources(chromiumResources);
  await loadScript('/resources/chromium/mock-idle-detection.js');
}
