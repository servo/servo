// META: script=/common/get-host-info.sub.js

// HTML PR https://github.com/whatwg/html/pull/8198 adds a definition for the
// HostEnsureCanAddPrivateElement host hook which disallows private fields on
// WindowProxy and Location objects.
//
// This test case ensure the hook works as designed.

let host_info = get_host_info();

const path = location.pathname.substring(0, location.pathname.lastIndexOf('/')) + '/frame.html';
const path_setdomain = path + "?setdomain";

class Base {
    constructor(o) {
        return o;
    }
}

class Stamper extends Base {
    #x = 10;
    static hasX(o) { return #x in o; }
};

function test_iframe_window(a_src, b_src) {
    const iframe = document.body.appendChild(document.createElement("iframe"));

    var resolve, reject;
    var promise = new Promise((res, rej) => {
        resolve = res;
        reject = rej
    });

    iframe.src = a_src;
    iframe.onload = () => {
        const windowA = iframe.contentWindow;
        try {
            assert_throws_js(TypeError, () => {
                new Stamper(windowA);
            }, "Can't Stamp (maybe cross-origin) exotic WindowProxy");
            assert_equals(Stamper.hasX(windowA), false, "Didn't stamp on WindowProxy");
        } catch (e) {
            reject(e);
            return;
        }

        iframe.src = b_src;
        iframe.onload = () => {
            const windowB = iframe.contentWindow;
            try {
                assert_equals(windowA == windowB, true, "Window is same")
                assert_throws_js(TypeError, () => {
                    new Stamper(windowA);
                }, "Can't Stamp (maybe cross-origin) exotics on WindowProxy");
                assert_equals(Stamper.hasX(windowB), false, "Didn't stamp on WindowProxy");
            } catch (e) {
                reject(e);
                return;
            }
            resolve();
        }
    };

    return promise;
}


function test_iframe_location(a_src, b_src) {
    const iframe = document.body.appendChild(document.createElement("iframe"));

    var resolve, reject;
    var promise = new Promise((res, rej) => {
        resolve = res;
        reject = rej
    });

    iframe.src = a_src;
    iframe.onload = () => {
        const locA = iframe.contentWindow.location;
        try {
            assert_throws_js(TypeError, () => {
                new Stamper(locA);
            }, "Can't Stamp (maybe cross-origin) exotic Location");
            assert_equals(Stamper.hasX(locA), false, "Didn't stamp on Location");
        } catch (e) {
            reject(e);
            return;
        }

        iframe.src = b_src;
        iframe.onload = () => {
            const locB = iframe.contentWindow.location
            try {
                assert_throws_js(TypeError, () => {
                    new Stamper(locB);
                }, "Can't Stamp cross-origin exotic Location");
                assert_equals(Stamper.hasX(locB), false, "Didn't stamp on Location");
            } catch (e) {
                reject(e);
                return;
            }
            resolve();
        }
    };

    return promise;
}

promise_test(() => test_iframe_window(host_info.HTTP_ORIGIN, host_info.HTTP_ORIGIN), "Same Origin: WindowProxy")
promise_test(() => test_iframe_window(host_info.HTTP_ORIGIN, host_info.HTTP_ORIGIN_WITH_DIFFERENT_PORT), "Cross Origin (port): WindowProxy")
promise_test(() => test_iframe_window(host_info.HTTP_ORIGIN, host_info.HTTP_REMOTE_ORIGIN), "Cross Origin (remote): WindowProxy")
promise_test(() => test_iframe_window(path, path_setdomain), "Same Origin + document.domain WindowProxy")


promise_test(() => test_iframe_location(host_info.HTTP_ORIGIN, host_info.HTTP_ORIGIN), "Same Origin: Location")
promise_test(() => test_iframe_location(host_info.HTTP_ORIGIN, host_info.HTTP_ORIGIN_WITH_DIFFERENT_PORT), "Cross Origin (remote): Location")
promise_test(() => test_iframe_location(host_info.HTTP_ORIGIN, host_info.HTTP_REMOTE_ORIGIN), "Cross Origin: Location")
promise_test(() => test_iframe_location(path, path_setdomain), "Same Origin + document.domain: Location")

// We can do this because promise_test promises to queue tests
// https://web-platform-tests.org/writing-tests/testharness-api.html#promise-tests

promise_test(async () => document.domain = document.domain, "Set document.domain");

promise_test(() => test_iframe_location(path, path_setdomain), "(After document.domain set) Same Origin + document.domain: Location")
promise_test(() => test_iframe_window(path, path_setdomain), "(After document.domain set) Same Origin + document.domain WindowProxy does carry private fields after navigation")

promise_test(() => test_iframe_location(path_setdomain, path_setdomain), "(After document.domain set) Local navigation (setdomain) Location")
promise_test(() => test_iframe_window(path_setdomain, path_setdomain), "(After document.domain set) Local navigation (setdomain) WindowProxy does carry private fields after navigation")
