// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

const params = new URLSearchParams();
params.set("description",
    "An early hints preload without `as` attribute should be ignored.");
params.set("resource-url",
    SAME_ORIGIN_RESOURCES_URL + "/empty.js?" + token());
params.set("should-preload", false);
const test_url = "resources/preload-as-test.h2.py?" + params.toString();
fetch_tests_from_window(openWindow(new URL(test_url, window.location)));
