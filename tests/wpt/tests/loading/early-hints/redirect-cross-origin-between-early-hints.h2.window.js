// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

test(() => {
    const params = new URLSearchParams();
    params.set("preload-before-redirect", SAME_ORIGIN_RESOURCES_URL + "/empty.js?" + token());
    params.set("preload-after-redirect", CROSS_ORIGIN_RESOURCES_URL + "/empty.js?" + token());
    params.set("redirect-url", CROSS_ORIGIN_RESOURCES_URL + "/redirect-between-early-hints.h2.py");
    params.set("final-test-page", "redirect-cross-origin-between-early-hints.html");

    params.set("test-step", "redirect");
    const test_url = "resources/redirect-between-early-hints.h2.py?" + params.toString();
    window.location.replace(new URL(test_url, window.location));
});