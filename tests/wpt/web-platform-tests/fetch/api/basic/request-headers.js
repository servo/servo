if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("../resources/utils.js");
}

function requestHeaders(desc, url, method, body, expectedOrigin, expectedContentLength) {
  var urlParameters = "?headers=origin|user-agent|accept-charset|content-length";
  var requestInit = {"method": method}
  if (body)
    requestInit["body"] = body;
  promise_test(function(test){
    return fetch(url + urlParameters, requestInit).then(function(resp) {
      assert_equals(resp.status, 200, "HTTP status is 200");
      assert_equals(resp.type , "basic", "Response's type is basic");
      assert_equals(resp.headers.get("x-request-origin") , expectedOrigin, "Request has header origin: " + expectedOrigin);
      assert_equals(resp.headers.get("x-request-content-length") , expectedContentLength, "Request has header content-length: " + expectedContentLength);
      assert_true(resp.headers.has("x-request-user-agent"), "Request has header user-agent");
      assert_false(resp.headers.has("accept-charset"), "Request has header accept-charset");
    });
  }, desc);
}

var url = RESOURCES_DIR + "inspect-headers.py"

requestHeaders("Fetch with GET", url, "GET", null, location.origin, null);
requestHeaders("Fetch with HEAD", url, "HEAD", null, location.origin, "0");
requestHeaders("Fetch with HEAD with body", url, "HEAD", "Request's body", location.origin, "14");
requestHeaders("Fetch with PUT without body", url, "POST", null, location.origin, "0");
requestHeaders("Fetch with PUT with body", url, "PUT", "Request's body", location.origin, "14");
requestHeaders("Fetch with POST without body", url, "POST", null, location.origin, "0");
requestHeaders("Fetch with POST with body", url, "POST", "Request's body", location.origin, "14");
requestHeaders("Fetch with Chicken", url, "Chicken", null, location.origin, null);
requestHeaders("Fetch with Chicken with body", url, "Chicken", "Request's body", location.origin, "14");

done();
