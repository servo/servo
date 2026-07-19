// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

test(() => {
    // A deliberately wrong digest for `empty.js`. The Early Hints Link header
    // is still parsed (so the resource is preloaded), but subresource
    // integrity must still be enforced when the resource is consumed.
    const WRONG_HASH = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    const preloads = [
        {
            "url": "empty.js?" + token(),
            "as_attr": "script",
            "integrity_attr": WRONG_HASH,
        },
    ];
    navigateToTestWithEarlyHints(
        "resources/preload-integrity-mismatch.html", preloads);
});
