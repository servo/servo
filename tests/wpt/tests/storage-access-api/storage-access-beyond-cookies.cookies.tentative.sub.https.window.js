// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

'use strict';

// Here's the set-up for this test:
// Step 1 (top-frame) Set up listener for "HasAccess" message.
// Step 2 (top-frame) Add data to first-party cookies.
// Step 3 (top-frame) Embed an iframe that's cross-site with top-frame.
// Step 4 (sub-frame) Try to use storage access API and read first-party data.
// Step 5 (sub-frame) Embed an iframe that's same-origin with top-frame.
// Step 6 (sub-sub-frame) Try to use storage access API and read first-party data.
// Step 7 (sub-sub-frame) Send "HasAccess for cookies" message to top-frame.
// Step 8 (top-frame) Cleanup.

async_test(t => {
  // Step 1
  window.addEventListener("message", t.step_func((e) => {
    if (e.data.type != "result") {
      return;
    }
    // Step 8
    assert_equals(e.data.message, "HasAccess for cookies", "Storage Access API should be accessible and return first-party data");
    t.add_cleanup(() => {test_driver.delete_all_cookies();});
    t.done();
  }));

  // Step 2
  const id = String(Date.now());
  document.cookie = "test=" + id + "; SameSite=None; Secure";

  // Step 3
  let iframe = document.createElement("iframe");
  iframe.src = "https://{{hosts[alt][]}}:{{ports[https][0]}}/storage-access-api/resources/storage-access-beyond-cookies-iframe.sub.html?type=cookies&id="+id;
  document.body.appendChild(iframe);
}, "Verify StorageAccessAPIBeyondCookies for Cookies");
