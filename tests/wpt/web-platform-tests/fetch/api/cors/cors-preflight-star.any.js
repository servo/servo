// META: script=../resources/utils.js
// META: script=/common/get-host-info.sub.js

const url = get_host_info().HTTP_REMOTE_ORIGIN + dirname(location.pathname) + RESOURCES_DIR + "preflight.py",
      origin = location.origin // assuming an ASCII origin

function preflightTest(succeeds, withCredentials, allowMethod, allowHeader, useMethod, useHeader) {
  return promise_test(t => {
    let testURL = url + "?",
        requestInit = {}
    if (withCredentials) {
      testURL += "origin=" + origin + "&"
      testURL += "credentials&"
      requestInit.credentials = "include"
    }
    if (useMethod) {
      requestInit.method = useMethod
    }
    if (useHeader) {
      requestInit.headers = [useHeader]
    }
    testURL += "allow_methods=" + allowMethod + "&"
    testURL += "allow_headers=" + allowHeader + "&"

    if (succeeds) {
      return fetch(testURL, requestInit).then(resp => {
        assert_equals(resp.headers.get("x-origin"), origin)
      })
    } else {
      return promise_rejects(t, new TypeError(), fetch(testURL, requestInit))
    }
  }, "CORS that " + (succeeds ? "succeeds" : "fails") + " with credentials: " + withCredentials + "; method: " + useMethod + " (allowed: " + allowMethod + "); header: " + useHeader + " (allowed: " + allowHeader + ")")
}

preflightTest(true, false, "get", "x-test", "GET", ["X-Test", "1"])
preflightTest(true, false, "*", "x-test", "SUPER", ["X-Test", "1"])
preflightTest(true, false, "*", "*", "OK", ["X-Test", "1"])
preflightTest(false, true, "*", "*", "OK", ["X-Test", "1"])
preflightTest(false, true, "*", "", "PUT", undefined)
preflightTest(false, true, "put", "*", "PUT", undefined)
preflightTest(false, true, "get", "*", "GET", ["X-Test", "1"])
preflightTest(false, true, "*", "*", "GET", ["X-Test", "1"])
