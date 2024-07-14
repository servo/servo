// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

'use strict';

// Here's the set-up for this test:
// Step 1 (top-frame) Set up listener for "HasAccess" message.
// Step 2 (top-frame) Write first-party cache storage.
// Step 3 (top-frame) Embed an iframe that's cross-site with top-frame.
// Step 4 (sub-frame) Try to use storage access API and read first-party data.
// Step 5 (sub-frame) Embed an iframe that's same-origin with top-frame.
// Step 6 (sub-sub-frame) Try to use storage access API and read first-party data.
// Step 7 (sub-sub-frame) Send "HasAccess for caches" message to top-frame.
// Step 8 (top-frame) Receive "HasAccess for caches" message and cleanup.

async_test(t => {
  // Step 1
  window.addEventListener("message", t.step_func((e) => {
    if (e.data.type != "result") {
      return;
    }
    // Step 8
    assert_equals(e.data.message, "HasAccess for caches", "Storage Access API should be accessible and return first-party data");
    t.add_cleanup(() => {test_driver.delete_all_cookies();});
    t.done();
  }));

  // Step 2
  const id = Date.now();
  document.cookie = "samesite_strict=test; SameSite=Strict; Secure";
  document.cookie = "samesite_lax=test; SameSite=Lax; Secure";
  document.cookie = "samesite_none=test; SameSite=None; Secure";

  window.caches.open(id).then(async (cache) => {
    await cache.add("https://{{hosts[][]}}:{{ports[https][0]}}/storage-access-api/resources/get_cookies.py?1");
    // Step 3
    let iframe = document.createElement("iframe");
    iframe.src = "https://{{hosts[alt][]}}:{{ports[https][0]}}/storage-access-api/resources/storage-access-beyond-cookies-iframe.sub.html?type=caches&id="+id;
    document.body.appendChild(iframe);
  });
}, "Verify StorageAccessAPIBeyondCookies for Cache Storage");
