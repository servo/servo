// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

'use strict';

async_test(t => {
  // Set up a message listener that simply calls t.done() when a message is received.
  window.addEventListener("message", t.step_func(e => {
    if (e.data.type != "result") {
      return;
    }
    assert_equals(e.data.message, "Blob URL DedicatedWorker tests completed successfully.");
    t.done();
  }));

  // Create an iframe that's cross-site with top-frame.
  const id = Date.now();
  let iframe = document.createElement("iframe");
  iframe.src = "https://{{hosts[alt][]}}:{{ports[https][0]}}/storage-access-api/resources/storage-access-beyond-cookies-iframe.sub.html?type=BlobURLDedicatedWorker&id=" + id;
  document.body.appendChild(iframe);

}, "Verify that if the third-party context creates a blob URL using the StorageAccessHandle and sends it to the dedicated worker, the dedicated worker fetch succeeds.");