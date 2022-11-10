// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

promise_test(async (t) => {
    const iframe = document.createElement("iframe");

    const resource_url = SAME_ORIGIN_RESOURCES_URL + "/empty.js?" + token();
    const params = new URLSearchParams();
    params.set("resource-url", resource_url);
    params.set("token", token());
    params.set("x-frame-options", "DENY");
    const iframe_url = SAME_ORIGIN_RESOURCES_URL + "/html-with-early-hints.h2.py?" + params.toString();

    iframe.src = iframe_url;
    document.body.appendChild(iframe);
    // Make sure the iframe didn't load. See https://github.com/whatwg/html/issues/125 for why a
    // timeout is used here. Long term all network error handling should be similar and have a
    // reliable event.
    assert_equals(iframe.contentDocument.body.localName, "body");
    await t.step_wait(() => iframe.contentDocument === null);

    // Fetch the hinted resource and make sure it's not preloaded.
    await fetchScript(resource_url);
    const entries = performance.getEntriesByName(resource_url);
    assert_equals(entries.length, 1);
    assert_not_equals(entries[0].transferSize, 0);
}, "Early hints for an iframe that violates X-Frame-Options should be ignored.");
