if (this.document === undefined) {
    importScripts("/resources/testharness.js");
    importScripts("../resources/utils.js");
    importScripts("../resources/get-host-info.sub.js")
}

function runTest(url, init, expectedReferrer, title) {
    promise_test(function(test) {
        return fetch(url , init).then(function(resp) {
            assert_equals(resp.status, 200, "HTTP status is 200");
            assert_equals(resp.headers.get("x-request-referer"), expectedReferrer, "Request's referrer is correct");
        });
    }, title);
}

var fetchedUrl = RESOURCES_DIR + "inspect-headers.py?headers=referer";
var corsFetchedUrl = get_host_info().HTTP_REMOTE_ORIGIN  + dirname(location.pathname) + RESOURCES_DIR + "inspect-headers.py?headers=referer&cors";

runTest(fetchedUrl, { referrerPolicy: "origin-when-cross-origin" }, location.toString(), "origin-when-cross-origin policy on a same-origin URL");
runTest(corsFetchedUrl, { referrerPolicy: "origin-when-cross-origin" }, get_host_info().HTTP_ORIGIN + "/", "origin-when-cross-origin policy on a cross-origin URL");

done();

