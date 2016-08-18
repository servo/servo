if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("../resources/utils.js");
}

function checkFetchResponse(url, method, desc) {
  if (!desc) {
    var cut = (url.length >= 40) ? "[...]" : "";
    cut += " (" + method + ")"
    desc = "Fetching " + url.substring(0, 40) + cut + " is OK"
  }
  promise_test(function(test) {
    return fetch(url, { method: method }).then(function(resp) {
     assert_equals(resp.status, 200, "HTTP status is 200");
     assert_equals(resp.type, "basic", "response type is basic");
     assert_equals(resp.headers.get("Content-Type"), "text/html;charset=utf-8", "Content-Type is " + resp.headers.get("Content-Type"));
     return resp.text();
    })
  }, desc);
}

checkFetchResponse("about:blank", "GET");
checkFetchResponse("about:blank", "PUT");
checkFetchResponse("about:blank", "POST");

function checkKoUrl(url, desc) {
  if (!desc)
    desc = "Fetching " + url.substring(0, 45) + " is KO"
  promise_test(function(test) {
    var promise = fetch(url);
    return promise_rejects(test, new TypeError(), promise);
  }, desc);
}

checkKoUrl("about:invalid.com");
checkKoUrl("about:config");
checkKoUrl("about:unicorn");

done();
