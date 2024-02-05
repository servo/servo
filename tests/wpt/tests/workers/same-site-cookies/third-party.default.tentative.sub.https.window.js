// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

'use strict';

// Here's the set-up for this test:
// Step 1 (top-frame) Set up listener for "DidStart" message and open cross-site iframe.
// Step 2 (sub-frame) Open iframe same-site to top-frame.
// Step 3 (sub-sub-frame) Set up listener for message and start worker.
// Step 4 (worker) Send "DidStart" message to iframe.
// Step 5 (sub-sub-frame) Receive message and pass on to window.
// Step 6 (top-frame) Receive "DidStart" message and cleanup.

async_test(t => {
  // Step 1
  window.addEventListener("message", t.step_func(e => {
    // Step 6
    assert_equals(e.data, "DidStart", "Worker should have started");
    t.done();
  }));
  let iframe = document.createElement("iframe");
  iframe.src = "https://{{hosts[alt][]}}:{{ports[https][0]}}/workers/same-site-cookies/resources/iframe.sub.html?type=default";
  document.body.appendChild(iframe);
}, "Check SharedWorker sameSiteCookies option default for third-party");
