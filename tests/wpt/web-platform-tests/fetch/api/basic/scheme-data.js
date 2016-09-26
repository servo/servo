if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("../resources/utils.js");
}

function checkFetchResponse(url, data, mime, fetchMode) {
  var cut = (url.length >= 40) ? "[...]" : "";
  desc = "Fetching " + url.substring(0, 40) + cut + " is OK";
  var init = { };
  if (fetchMode) {
    init.mode = fetchMode;
    desc += " (" + fetchMode + ")";
  }
  promise_test(function(test) {
    return fetch(url, init).then(function(resp) {
      assert_equals(resp.status, 200, "HTTP status is 200");
      assert_equals(resp.statusText, "OK", "HTTP statusText is OK");
      assert_equals(resp.type, "basic", "response type is basic");
      assert_equals(resp.headers.get("Content-Type"), mime, "Content-Type is " + resp.headers.get("Content-Type"));
     return resp.text();
    }).then(function(body) {
        assert_equals(body, data, "Response's body is correct");
    });
  }, desc);
}

checkFetchResponse("data:,response%27s%20body", "response's body", "text/plain;charset=US-ASCII");
checkFetchResponse("data:,response%27s%20body", "response's body", "text/plain;charset=US-ASCII", "same-origin");
checkFetchResponse("data:,response%27s%20body", "response's body", "text/plain;charset=US-ASCII", "cors");
checkFetchResponse("data:text/plain;base64,cmVzcG9uc2UncyBib2R5", "response's body", "text/plain");
checkFetchResponse("data:image/png;base64,cmVzcG9uc2UncyBib2R5",
                   "response's body",
                   "image/png");

function checkKoUrl(url, method, desc) {
  var cut = (url.length >= 40) ? "[...]" : "";
  desc = "Fetching [" + method + "] " + url.substring(0, 45) + cut + " is KO"
  promise_test(function(test) {
    return promise_rejects(test, new TypeError(), fetch(url, {"method": method}));
  }, desc);
}

checkKoUrl("data:notAdataUrl.com", "GET");
checkKoUrl("data:,response%27s%20body", "POST");
checkKoUrl("data:,response%27s%20body", "HEAD");

done();
