// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

'use strict';

// Here's the set-up for this test:
// Step 1 (window) Set up listener for "DidStart" message and start worker.
// Step 2 (worker) Send "DidStart" message to window.
// Step 3 (window) Receive "DidStart" message and cleanup.

async_test(t => {
    // Step 1
    const worker = new SharedWorker("/workers/same-site-cookies/resources/worker.js", {sameSiteCookies: "all"});
    worker.port.onmessage = t.step_func(e => {
        // Step 3
        assert_equals(e.data, "DidStart", "Worker should have started");
        t.done();
    });
}, "Check SharedWorker sameSiteCookies option all for first-party");
