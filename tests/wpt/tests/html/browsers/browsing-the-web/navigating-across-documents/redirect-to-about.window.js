"use strict";

["about:blank", "about:srcdoc", "about:nonstandard"].forEach(aboutURL => {
  promise_test(async t => {
    const iframe = document.createElement("iframe");
    iframe.src = `resources/redirect.py?location=${aboutURL}`;
    document.body.append(iframe);

    // Unfortunately Firefox does not fire a load event for network errors yet, but there is no
    // other way I can see to test this. (Also applicable below.)
    await new Promise(r => iframe.onload = r);

    // Must throw since error pages are opaque origin.
    assert_throws_dom("SecurityError", () => {
      iframe.contentWindow.document;
    });
  }, `An iframe with src set to a redirect to ${aboutURL}`);

  promise_test(async t => {
    const iframe = document.createElement("iframe");
    iframe.src = "/common/blank.html";
    document.body.append(iframe);

    await new Promise(r => iframe.onload = r);

    iframe.contentWindow.location.href = `resources/redirect.py?location=${aboutURL}`;
    await new Promise(r => iframe.onload = r);

    // Must throw since error pages are opaque origin.
    assert_throws_dom("SecurityError", () => {
      iframe.contentWindow.document;
    });
  }, `An iframe that is navigated to a redirect to ${aboutURL}`);
});
