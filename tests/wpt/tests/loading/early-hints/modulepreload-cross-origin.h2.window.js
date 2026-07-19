// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

// see modulepreload-in-early-hints.h2.window.js for params explanation
const params = new URLSearchParams();
params.set("description", "Modulepreload works in early hints from cross-origin url");
params.set("resource-url",
    CROSS_ORIGIN_RESOURCES_URL + "/empty.js?" + token());
params.set("should-preload", true);
const test_url = "resources/modulepreload-in-early-hints.h2.py?" + params.toString();
fetch_tests_from_window(openWindow(new URL(test_url, window.location)));
