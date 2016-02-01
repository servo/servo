if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("/common/utils.js");
  importScripts("../resources/utils.js");
}

function corsRedirect(desc, redirectUrl, redirectLocation, redirectStatus, expectedOrigin) {
  var uuid_token = token();
  var url = redirectUrl;
  var urlParameters = "?token=" + uuid_token + "&max_age=0";
  urlParameters += "&redirect_status=" + redirectStatus;
  urlParameters += "&location=" + encodeURIComponent(redirectLocation);

  var requestInit = {"mode": "cors", "redirect": "follow"};

  promise_test(function(test) {
    fetch(RESOURCES_DIR + "clean-stash.py?token=" + uuid_token).then(function(resp) {
      return fetch(url + urlParameters, requestInit).then(function(resp) {
        assert_equals(resp.status, 200, "Response's status is 200");
        assert_equals(resp.headers.get("x-did-preflight"), "0", "No preflight request has been made");
        assert_equals(resp.headers.get("x-origin"), expectedOrigin, "Origin is correctly set after redirect");
      });
    });
  }, desc);
}

var redirPath = dirname(location.pathname) + RESOURCES_DIR + "redirect.py";
var preflightPath = dirname(location.pathname) + RESOURCES_DIR + "preflight.py";

var localRedirect = "http://{{host}}:{{ports[http][0]}}" + redirPath;
var remoteRedirect = "http://www1.{{host}}:{{ports[http][0]}}" + redirPath;

var localLocation = "http://{{host}}:{{ports[http][0]}}" + preflightPath;
var remoteLocation = "http://www1.{{host}}:{{ports[http][0]}}" + preflightPath;
var remoteLocation2 = "http://www.{{host}}:{{ports[http][0]}}" + preflightPath;

for (var code of [301, 302, 303, 307, 308]) {
  corsRedirect("Redirect " + code + ": cors to same cors", remoteRedirect, remoteLocation, code, location.origin);
  corsRedirect("Redirect " + code + ": cors to another cors", remoteRedirect, remoteLocation2, code, "null");
  corsRedirect("Redirect " + code + ": same origin to cors", localRedirect, remoteLocation, code, location.origin);
  corsRedirect("Redirect " + code + ": cors to same origin", remoteRedirect, localLocation, code, "null");
}

done();
