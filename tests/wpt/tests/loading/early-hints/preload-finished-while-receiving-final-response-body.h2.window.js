// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

test(() => {
    const params = new URLSearchParams();
    const id = token();
    params.set("resource-url", SAME_ORIGIN_RESOURCES_URL + "/fetch-and-record-js.h2.py?id=" + id);
    params.set("resource-id", id);
    const test_url = "resources/preload-finished-while-receiving-final-response-body.h2.py?" + params.toString();
    window.location.replace(new URL(test_url, window.location));
});