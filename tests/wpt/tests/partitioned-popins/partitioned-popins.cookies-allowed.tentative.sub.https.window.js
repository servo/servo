// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/storage-access-api/helpers.js

'use strict';

// Spec: https://explainers-by-googlers.github.io/partitioned-popins/
// Step 1 (main-window) Set up listener to resolve messages as they come in.
// Step 2 (main-window) Open window for other origin.
// Step 3 (other-window) Write first-party cookies and report success.
// Step 4 (main-window) Embed iframe for other origin.
// Step 5 (main-iframe) Write third-party cookies and report success.
// Step 6 (main-window) Open partitioned popin for other origin.
// Step 7 (popin) Check for first-/third-party cookies.
// Step 8 (popin-iframe) Check for first-/third-party cookies and report success.
// Step 9 (popin) Report success.
// Step 10 (main-window) Cleanup.

async_test(t => {
  const id = String(Math.random());
  document.cookie = "FirstPartyStrict=" + id + "; SameSite=Strict; Secure";
  document.cookie = "FirstPartyLax=" + id + "; SameSite=Lax; Secure";
  document.cookie = "FirstPartyNone=" + id + "; SameSite=None; Secure";
  // Step 1
  window.addEventListener("message", t.step_func(e => {
    switch (e.data.type) {
      case 'window-set':
        // Step 4
        assert_equals(e.data.message, "Set first-party data");
        const iframe = document.createElement("iframe");
        iframe.src = "https://{{hosts[alt][]}}:{{ports[https][0]}}/partitioned-popins/resources/partitioned-popins.cookies-iframe.html?id="+id;
        document.body.appendChild(iframe);
        break;
      case 'iframe-set':
        // Step 6
        assert_equals(e.data.message, "Set third-party data");
        window.open("https://{{hosts[alt][]}}:{{ports[https][0]}}/partitioned-popins/resources/partitioned-popins.cookies-popin.sub.py?id="+id, '_blank', 'popin');
        break;
      case 'popin-read':
        // Step 10
        // We want to see the same behavior a cross-site iframe would have, only SameSite=None available, with the ability to set additional cookies in the popin.
        assert_equals(e.data.message, "ReadOnLoad:FirstPartyNone-ThirdPartyNone-,ReadOnFetch:FirstPartyNone-ThirdPartyNone-FirstPartyNonePopin-ThirdPartyNonePopin-,ReadOnDocument:FirstPartyNone-ThirdPartyNone-FirstPartyNonePopin-ThirdPartyNonePopin-,ReadOnFetchAfterRSA:FirstPartyNone-ThirdPartyNone-FirstPartyNonePopin-ThirdPartyNonePopin-FirstPartyNonePopinAfterRSA-ThirdPartyNonePopinAfterRSA-,ReadOnDocumentAfterRSA:FirstPartyNone-ThirdPartyNone-FirstPartyNonePopin-ThirdPartyNonePopin-FirstPartyNonePopinAfterRSA-ThirdPartyNonePopinAfterRSA-,ReadInPopinIframe:FirstPartyNone-ThirdPartyNone-,ReadInPopinIframeAfterRSA:FirstPartyNone-ThirdPartyNone-FirstPartyNoneAfterRSA-ThirdPartyNoneAfterRSA-");
        t.done();
        break;
    }
  }));

  MaybeSetStorageAccess("*", "*", "allowed").then(() => {
    // Step 2
    window.open("https://{{hosts[alt][]}}:{{ports[https][0]}}/partitioned-popins/resources/partitioned-popins.cookies-window.html?id="+id, '_blank', 'popup');
  });
}, "Verify Partitioned Popins cookie access when third-party cookie access allowed");
