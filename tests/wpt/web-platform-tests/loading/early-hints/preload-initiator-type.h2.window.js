// META: script=resources/early-hints-helpers.sub.js

test(() => {
    const preloads = [{
        "url": "empty.js?" + Date.now(),
        "as_attr": "script",
    }];
    navigateToTestWithEarlyHints("resources/preload-initiator-type.html", preloads);
});