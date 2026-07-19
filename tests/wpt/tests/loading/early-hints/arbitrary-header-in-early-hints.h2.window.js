// META: script=resources/early-hints-helpers.sub.js
const test_url = "resources/arbitrary-header-in-early-hints.h2.py";
fetch_tests_from_window(openWindow(new URL(test_url, window.location)));
