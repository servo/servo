// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

test(() => {
    const params = new URLSearchParams();
    params.set("preload-url", SAME_ORIGIN_RESOURCES_URL + "/empty.js?" + token());
    params.set("redirect-url", CROSS_ORIGIN_RESOURCES_URL + "/redirect-cross-origin.html");
    const test_url = "resources/redirect-with-early-hints.h2.py?" + params.toString();
    window.location.replace(new URL(test_url, window.location));
});
