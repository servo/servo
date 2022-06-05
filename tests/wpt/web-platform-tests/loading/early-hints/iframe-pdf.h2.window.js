// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

promise_test(async (t) => {
    if (!navigator.pdfViewerEnabled) {
        return;
    }

    const iframe = document.createElement("iframe");
    const resource_url = SAME_ORIGIN_RESOURCES_URL + "/empty.js?" + token();
    const promise = new Promise((resolve) => {
        const params = new URLSearchParams();
        params.set("resource-url", resource_url);
        params.set("token", token());
        const iframe_url = SAME_ORIGIN_RESOURCES_URL + "/pdf-with-early-hints.h2.py?" + params.toString();

        iframe.src = iframe_url;
        iframe.onload = resolve;
        document.body.appendChild(iframe);
    });
    await promise;

    // `iframe` should not preload the hinted resource.
    const iframe_entries = iframe.contentWindow.performance.getEntriesByName(resource_url);
    assert_equals(iframe_entries.length, 0);

    await fetchScript(resource_url);
    const entries = performance.getEntriesByName(resource_url);
    assert_equals(entries.length, 1);
    assert_not_equals(entries[0].transferSize, 0);
}, "Early hints for an iframe of which content is pdf should be ignored.");
