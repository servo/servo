// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

const params = new URLSearchParams();
params.set("preload-before-redirect", SAME_ORIGIN_RESOURCES_URL + "/empty.js?" + token());
params.set("preload-after-redirect", SAME_ORIGIN_RESOURCES_URL + "/empty.js?" + token());
params.set("redirect-url", SAME_ORIGIN_RESOURCES_URL + "/redirect-between-early-hints.h2.py");
params.set("final-test-page", "redirect-same-origin-between-early-hints.html");

params.set("test-step", "redirect");
const test_url = "resources/redirect-between-early-hints.h2.py?" + params.toString();
fetch_tests_from_window(openWindow(new URL(test_url, window.location)));
