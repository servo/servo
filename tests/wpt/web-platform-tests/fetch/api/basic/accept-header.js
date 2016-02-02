if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("../resources/utils.js");
}

promise_test(function() {
  return fetch(RESOURCES_DIR + "inspect-headers.py?headers=Accept").then(function(response) {
    assert_equals(response.status, 200, "HTTP status is 200");
    assert_equals(response.type , "basic", "Response's type is basic");
    assert_equals(response.headers.get("x-request-accept"), "*/*", "Request has accept header with value '*/*'");
  });
}, "Request through fetch should have 'accept' header with value '*/*'");

done();
