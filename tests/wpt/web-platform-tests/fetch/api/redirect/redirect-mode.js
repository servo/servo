if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("../resources/utils.js");
}

function redirectMode(desc, redirectUrl, redirectLocation, redirectStatus, redirectMode) {
  var url = redirectUrl;
  var urlParameters = "?redirect_status=" + redirectStatus;
  urlParameters += "&location=" + encodeURIComponent(redirectLocation);

  var requestInit = {"redirect": redirectMode};

  promise_test(function(test) {
    if (redirectMode === "error")
      return promise_rejects(test, new TypeError(), fetch(url + urlParameters, requestInit));
    if (redirectMode === "manual")
      return fetch(url + urlParameters, requestInit).then(function(resp) {
        assert_equals(resp.status, 0, "Response's status is 0");
        assert_equals(resp.type, "opaqueredirect", "Response's type is opaqueredirect");
        assert_equals(resp.statusText, "", "Response's statusText is \"\"");
      });
    if (redirectMode === "follow")
      return fetch(url + urlParameters, requestInit).then(function(resp) {
        assert_true(new URL(resp.url).pathname.endsWith(locationUrl), "Response's url should be the redirected one");
        assert_equals(resp.status, 200, "Response's status is 200");
      });
    assert_unreached(redirectMode + " is no a valid redirect mode");
  }, desc);
}

var redirUrl = RESOURCES_DIR + "redirect.py";
var locationUrl = "top.txt";

for (var statusCode of [301, 302, 303, 307, 308]) {
  redirectMode("Redirect " + statusCode + " in \"error\" mode ", redirUrl, locationUrl, statusCode, "error");
  redirectMode("Redirect " + statusCode + " in \"follow\" mode ", redirUrl, locationUrl, statusCode, "follow");
  redirectMode("Redirect " + statusCode + " in \"manual\" mode ", redirUrl, locationUrl, statusCode, "manual");
}

done();
