// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

'use strict';

// Here's the set-up for this test:
// Step 1 (top-frame) Set up fallback failure listener for if the handle cannot be used.
// Step 2 (top-frame) Set up relay worker to expect "Same-origin handle access".
// Step 3 (top-frame) Set cookies and embed an iframe that's cross-site with top-frame.
// Step 4 (sub-frame) Try to use storage access API to access shared worker.
// Step 5 (sub-frame) Embed an iframe that's same-origin with top-frame.
// Step 6 (sub-sub-frame) Try to use storage access API to access first-party shared worker.
// Step 7 (sub-sub-frame) Send "HasAccess for SharedWorker" message to top-frame.
// Step 8 (top-frame) Set up cookie worker to expect it's already opened.

async_test(t => {
  // Step 1
  window.addEventListener("message", t.step_func(e => {
    if (e.data.type != "result") {
      return;
    }
    assert_equals(e.data.message, "HasAccess for SharedWorker", "Storage Access API should be accessible and return first-party data");
  }));

  // Step 2
  const id = Date.now();
  const relay_worker = new SharedWorker("/storage-access-api/resources/shared-worker-relay.js", {name: id, sameSiteCookies: 'none'});
  relay_worker.port.onmessage = t.step_func(e => {
    assert_equals(e.data, "Same-origin handle access", "Relay worker should divert messages here");
    // Step 8
    const cookie_worker = new SharedWorker("/storage-access-api/resources/shared-worker-cookies.py", {name: id, sameSiteCookies: 'none'});
    cookie_worker.port.onmessage = t.step_func(async (e) => {
      assert_equals(e.data, "ReadOnLoad:None,ReadOnFetch:None,ConnectionsMade:2", "Worker should already have been opened and only see SameSite=None cookies");
      await test_driver.delete_all_cookies();
      t.done();
    });
  });

  // Step 3
  const cookie_set_window = window.open("/storage-access-api/resources/set_cookies.py");
  cookie_set_window.onload =  t.step_func(_ => {
    let iframe = document.createElement("iframe");
    iframe.src = "https://{{hosts[alt][]}}:{{ports[https][0]}}/storage-access-api/resources/storage-access-beyond-cookies-iframe.sub.html?type=SharedWorker&id="+id;
    document.body.appendChild(iframe);
  });
}, "Verify StorageAccessAPIBeyondCookies for Shared Worker");
