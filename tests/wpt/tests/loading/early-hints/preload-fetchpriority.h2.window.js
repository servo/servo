// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

const preloads = ["low", "high", "auto"].map(fetchpriority => { return {
    "url": "empty.js?" + token() + fetchpriority,
    "as_attr": "script",
    "fetchpriority_attr": fetchpriority,
}});
fetch_tests_from_window(navigateToTestWithEarlyHints("resources/preload-fetchpriority.html", preloads));
