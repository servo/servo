// META: script=resources/early-hints-helpers.sub.js

const preloads = [{
    "url": "empty.json?" + Date.now(),
    "as_attr": "fetch",
    "crossorigin_attr": "",
}];
fetch_tests_from_window(navigateToTestWithEarlyHints("resources/preload-fetch.html", preloads));
