// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

// params are sent to a Python handler[1] that returns a 103 Early Hints
// response based the values of "resource-url" and "as", and then that response
// is validated by a window test[2] according to the value of "should-preload"
//
// see: https://web-platform-tests.org/writing-tests/h2tests.html
//
// [1]: resources/modulepreload-in-early-hints.h2.py
// [2]: resources/modulepreload-in-early-hints.h2.html
test(() => {
    const params = new URLSearchParams();
    params.set("description", "Modulepreload works in early hints");
    params.set("resource-url",
        SAME_ORIGIN_RESOURCES_URL + "/empty.js?" + token());
    params.set("should-preload", true);
    const test_url = "resources/modulepreload-in-early-hints.h2.py?" + params.toString();
    window.location.replace(new URL(test_url, window.location));
});
