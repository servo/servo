// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

'use strict';

// Here's the set-up for this test:
// Step 1 (top-frame) Set up listener for "HasAccess" message.
// Step 2 (top-frame) Secure first-party web lock.
// Step 3 (top-frame) Embed an iframe that's cross-site with top-frame.
// Step 4 (sub-frame) Try to use storage access API and read first-party data.
// Step 5 (sub-frame) Embed an iframe that's same-origin with top-frame.
// Step 6 (sub-sub-frame) Try to use storage access API and read first-party data.
// Step 7 (sub-sub-frame) Send "HasAccess for locks" message to top-frame.
// Step 8 (top-frame) Receive "HasAccess for locks" message and cleanup.

async_test(t => {
  // Step 1
  window.addEventListener("message", t.step_func(e => {
    if (e.data.type != "result") {
      return;
    }
    // Step 8
    assert_equals(e.data.message, "HasAccess for locks", "Storage Access API should be accessible and return first-party data");
    t.done();
  }));

  // Step 2
  const id = Date.now();
  window.navigator.locks.request(id, async () => {
    // Step 3
    let iframe = document.createElement("iframe");
    iframe.src = "https://{{hosts[alt][]}}:{{ports[https][0]}}/storage-access-api/resources/storage-access-beyond-cookies-iframe.sub.html?type=locks&id="+id;
    document.body.appendChild(iframe);
    // We need to sleep so this locks stays engaged until the test succeeds or times out.
    await new Promise(r => setTimeout(r, 30000))
  }).catch (() => {/*We expect this lock to be stolen if the test runs correctly.*/});
}, "Verify StorageAccessAPIBeyondCookies for Web Locks");
