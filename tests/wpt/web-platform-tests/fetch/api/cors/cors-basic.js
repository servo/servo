if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("../resources/utils.js");
}

function cors(desc, scheme, subdomain, port) {
  if (!port)
    port = location.port;
  if (subdomain)
    subdomain = subdomain + ".";
  else
    subdomain = "";

  var url = scheme + "://" + subdomain + "{{host}}" + ":" + port + dirname(location.pathname);
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

cors("Cross domain basic usage", "http", "www1");
cors("Same domain different port", "http", undefined, "{{ports[http][1]}}");
cors("Cross domain different port", "http", "www1", "{{ports[http][1]}}");
cors("Cross domain different protocol", "https", "www1", "{{ports[https][0]}}");
cors("Same domain different protocol different port", "https", undefined, "{{ports[https][0]}}");

done();
