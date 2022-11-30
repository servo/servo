// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

test(() => {
    const params = new URLSearchParams();
    params.set("description",
        "An early hints preload with an invalid `as` attribute should be ignored.");
    params.set("resource-url",
        SAME_ORIGIN_RESOURCES_URL + "/empty.js?" + token());
    params.set("as", "invalid");
    params.set("should-preload", false);
    const test_url = "resources/preload-as-test.h2.py?" + params.toString();
    window.location.replace(new URL(test_url, window.location));
});
