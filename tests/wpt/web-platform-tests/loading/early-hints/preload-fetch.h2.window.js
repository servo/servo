// META: script=resources/early-hints-helpers.sub.js

test(() => {
    const preloads = [{
        "url": "empty.json?" + Date.now(),
        "as_attr": "fetch",
        "crossorigin_attr": "",
    }];
    navigateToTestWithEarlyHints("resources/preload-fetch.html", preloads);
});
