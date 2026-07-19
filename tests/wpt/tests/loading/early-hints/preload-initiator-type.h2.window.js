// META: script=resources/early-hints-helpers.sub.js

const preloads = [{
    "url": "empty.js?" + Date.now(),
    "as_attr": "script",
}];
fetch_tests_from_window(navigateToTestWithEarlyHints("resources/preload-initiator-type.html", preloads));
