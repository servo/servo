// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

'use strict';

async_test(t => {
  window.addEventListener("message", t.step_func(async e => {
    if (e.data.type !== "blobURL") {
      return;
    }
    const blob_url = e.data.message;

    // Create an iframe and pass the blob URL to it.
    const id = btoa(blob_url);
    const iframe = document.createElement("iframe");
    iframe.src = "https://{{hosts[alt][]}}:{{ports[https][0]}}/storage-access-api/resources/storage-access-beyond-cookies-iframe.sub.html?type=ThirdPartyBlobURL&id=" + id;
    document.body.appendChild(iframe);

    // Set up a second message listener to receive the result from the iframe.
    window.addEventListener("message", t.step_func(e => {
      if (e.data.type !== "result") {
        return;
      }
      assert_equals(e.data.message, "Third Party Blob URL tests completed successfully.");
      popup.close();
      t.done();
    }));
  }));

  // Open a popup to create the blob URL.
  const popup = window.open("https://{{hosts[alt][]}}:{{ports[https][0]}}/storage-access-api/resources/iframe-creation.sub.html");

}, "Verify StorageAccessAPIBeyondCookies for third-party context accessing first-party Blob URLs");
