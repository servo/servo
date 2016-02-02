if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("../resources/utils.js");
}

function redirectMethod(desc, redirectUrl, redirectLocation, redirectStatus, method, expectedMethod) {
  var url = redirectUrl;
  var urlParameters = "?redirect_status=" + redirectStatus;
  urlParameters += "&location=" + encodeURIComponent(redirectLocation);

  var requestInit = {"method": method, "redirect": "follow"};
  if (method != "GET" && method != "HEAD")
    requestInit.body = "this is my body";

  promise_test(function(test) {
    return fetch(url + urlParameters, requestInit).then(function(resp) {
      assert_equals(resp.status, 200, "Response's status is 200");
      assert_equals(resp.type, "basic", "Response's type basic");
      assert_equals(resp.headers.get("x-request-method"), expectedMethod, "Request method after redirection is " + expectedMethod);
      return resp.text().then(function(text) {
        assert_equals(text, expectedMethod == "POST" ? requestInit.body : "");
      });
    });
  }, desc);
}

var redirUrl = RESOURCES_DIR + "redirect.py";
var locationUrl = "method.py";

redirectMethod("Redirect 301 with GET", redirUrl, locationUrl, 301, "GET", "GET");
redirectMethod("Redirect 301 with POST", redirUrl, locationUrl, 301, "POST", "GET");
redirectMethod("Redirect 301 with HEAD", redirUrl, locationUrl, 301, "HEAD", "HEAD");

redirectMethod("Redirect 302 with GET", redirUrl, locationUrl, 302, "GET", "GET");
redirectMethod("Redirect 302 with POST", redirUrl, locationUrl, 302, "POST", "GET");
redirectMethod("Redirect 302 with HEAD", redirUrl, locationUrl, 302, "HEAD", "HEAD");

redirectMethod("Redirect 303 with GET", redirUrl, locationUrl, 303, "GET", "GET");
redirectMethod("Redirect 303 with POST", redirUrl, locationUrl, 303, "POST", "GET");
redirectMethod("Redirect 303 with HEAD", redirUrl, locationUrl, 303, "HEAD", "HEAD");

redirectMethod("Redirect 307 with GET", redirUrl, locationUrl, 307, "GET", "GET");
redirectMethod("Redirect 307 with POST", redirUrl, locationUrl, 307, "POST", "POST");
redirectMethod("Redirect 307 with HEAD", redirUrl, locationUrl, 307, "HEAD", "HEAD");

done();
