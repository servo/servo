if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("../resources/utils.js");
}

var origin = "http://{{host}}:{{ports[http][0]}}";
var fetchedUrl = RESOURCES_DIR + "inspect-headers.py?headers=referer";

promise_test(function(test) {
  return fetch(fetchedUrl).then(function(resp) {
    assert_equals(resp.status, 200, "HTTP status is 200");
    assert_equals(resp.type , "basic", "Response's type is basic");
    assert_equals(resp.headers.get("x-request-referer"), origin, "request's referrer is " + origin);
  });
}, "Request's referrer is origin");

promise_test(function(test) {
  var referrerUrl = "http://{{domains[www]}}:{{ports[http][0]}}/";
  return promise_rejects(test, new TypeError(), fetch(fetchedUrl, { "referrer":  referrerUrl}));
}, "Throw a TypeError referrer is not same-origin with origin");

done();
