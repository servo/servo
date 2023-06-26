// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

test(() => {
    const params = new URLSearchParams();
    params.set("resource-url",
        SAME_ORIGIN_RESOURCES_URL + "/square.png?" + token());
    const test_url = "resources/404-with-early-hints.h2.py?" + params.toString();
    window.location.replace(new URL(test_url, window.location));
});
