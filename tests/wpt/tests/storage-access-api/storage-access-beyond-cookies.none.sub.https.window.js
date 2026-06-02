// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

'use strict';

// Here's the set-up for this test:
// Step 1 (top-frame) Set up listener for "HasAccess" message.
// Step 2 (top-frame) Skipped in this test, but numbering must be consistent with other tests.
// Step 3 (top-frame) Embed an iframe that's cross-site with top-frame.
// Step 4 (sub-frame) Skipped in this test, but numbering must be consistent with other tests.
// Step 5 (sub-frame) Embed an iframe that's same-origin with top-frame.
// Step 6 (sub-sub-frame) Try to use storage access API without requesting anything.
// Step 7 (sub-sub-frame) Send "HasAccess for none" message to top-frame.
// Step 8 (top-frame) Cleanup.

async_test(t => {
  // Step 1
  window.addEventListener("message", t.step_func(e => {
    // Step 8
    if (e.data.type != "result") {
      return;
    }
    assert_equals(e.data.message, "HasAccess for none", "Storage Access API should not allow access for empty requests.");
    t.done();
  }));

  // Step 2
  // Step 3
  let iframe = document.createElement("iframe");
  iframe.src = "https://{{hosts[alt][]}}:{{ports[https][0]}}/storage-access-api/resources/storage-access-beyond-cookies-iframe.sub.html?type=none&id=";
  document.body.appendChild(iframe);
}, "Verify StorageAccessAPIBeyondCookies for None");
