if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("../resources/utils.js");
  importScripts("../resources/get-host-info.sub.js");
}

function cors(desc, origin) {
  var url = origin + dirname(location.pathname);
  var urlParameters = "?pipe=header(Access-Control-Allow-Origin,*)";

  promise_test(function(test) {
    return fetch(url + RESOURCES_DIR + "top.txt" + urlParameters, {"mode": "no-cors"} ).then(function(resp) {
      assert_equals(resp.status, 0, "Opaque filter: status is 0");
      assert_equals(resp.statusText, "", "Opaque filter: statusText is \"\"");
      assert_equals(resp.type , "opaque", "Opaque filter: response's type is opaque");
    });
  }, desc + " [no-cors mode]");

  promise_test(function(test) {
    var testedPromise = fetch(url + RESOURCES_DIR + "top.txt", {"mode": "cors"} ).then(function(resp) {
      return promise_rejects(test, new TypeError(), testedPromise);
    });
  }, desc + " [server forbid CORS]");

  promise_test(function(test) {
    return fetch(url + RESOURCES_DIR + "top.txt" + urlParameters, {"mode": "cors"} ).then(function(resp) {
      assert_equals(resp.status, 200, "Fetch's response's status is 200");
      assert_equals(resp.type , "cors", "CORS response's type is cors");
    });
  }, desc + " [cors mode]");
}

var host_info = get_host_info();

cors("Same domain different port", host_info.HTTP_ORIGIN_WITH_DIFFERENT_PORT);
cors("Same domain different protocol different port", host_info.HTTPS_ORIGIN);
cors("Cross domain basic usage", host_info.HTTP_REMOTE_ORIGIN);
cors("Cross domain different port", host_info.HTTP_REMOTE_ORIGIN_WITH_DIFFERENT_PORT);
cors("Cross domain different protocol", host_info.HTTPS_REMOTE_ORIGIN);

done();
