// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

test(() => {
    const resource_origin = CROSS_ORIGIN;
    const resource_url = CROSS_ORIGIN + RESOURCES_PATH + "/empty.js?" + token();
    const params = new URLSearchParams();
    params.set("resource-origin", resource_origin);
    params.set("resource-url", resource_url);
    const test_url = "resources/preconnect-in-early-hints.h2.py?" + params.toString();
    window.location.replace(new URL(test_url, window.location));
});
