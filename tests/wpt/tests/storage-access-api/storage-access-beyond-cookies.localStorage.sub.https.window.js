// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

'use strict';

// Here's the set-up for this test:
// Step 1 (top-frame) Set up listener for "HasAccess" message and set data for window_event if so.
// Step 2 (top-frame) Add data to first-party local storage and set up listener for handle_event.
// Step 3 (top-frame) Embed an iframe that's cross-site with top-frame.
// Step 4 (sub-frame) Try to use storage access API and read first-party data.
// Step 5 (sub-frame) Embed an iframe that's same-origin with top-frame.
// Step 6 (sub-sub-frame) Try to use storage access API and read first-party data.
// Step 7 (sub-sub-frame) Send "HasAccess for localStorage" message to top-frame.
// Step 8 (top-frame) Expect new data to be set and properly trigger listener.

async_test(t => {
  // Step 1
  window.addEventListener("message", t.step_func(e => {
    if (e.data.type != "result") {
      return;
    }
    assert_equals(e.data.message, "HasAccess for localStorage", "Storage Access API should be accessible and return first-party data");
    window.localStorage.setItem("window_event", id);
  }));

  // Step 2
  const id = String(Date.now());
  window.localStorage.setItem("test", id);
  window.addEventListener("storage", t.step_func(e => {
    if (e.key == "handle_event") {
      // Step 8
      assert_equals(e.newValue, id);
      assert_equals(e.storageArea, window.localStorage);
      window.localStorage.clear();
      t.done();
    }
  }));

  // Step 3
  let iframe = document.createElement("iframe");
  iframe.src = "https://{{hosts[alt][]}}:{{ports[https][0]}}/storage-access-api/resources/storage-access-beyond-cookies-iframe.sub.html?type=localStorage&id="+id;
  document.body.appendChild(iframe);
}, "Verify StorageAccessAPIBeyondCookies for Local Storage");
