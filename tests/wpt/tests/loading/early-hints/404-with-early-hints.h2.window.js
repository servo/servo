// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

const params = new URLSearchParams();
params.set("resource-url",
    SAME_ORIGIN_RESOURCES_URL + "/square.png?" + token());
const test_url = "resources/404-with-early-hints.h2.py?" + params.toString();
fetch_tests_from_window(openWindow(new URL(test_url, window.location)));
