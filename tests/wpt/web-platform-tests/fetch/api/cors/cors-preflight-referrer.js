if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("/common/utils.js");
  importScripts("../resources/utils.js");
}

function corsPreflightReferrer(desc, corsUrl, referrerPolicy, expectedReferrer) {
  var uuid_token = token();
  var url = corsUrl;
  var urlParameters = "?token=" + uuid_token + "&max_age=0";
  var requestInit = {"mode": "cors", "referrerPolicy": referrerPolicy};

  /* Force preflight */
  requestInit["headers"] = {"x-force-preflight": ""};
  urlParameters += "&allow_headers=x-force-preflight";

  promise_test(function(test) {
    return fetch(RESOURCES_DIR + "clean-stash.py?token=" + uuid_token).then(function(resp) {
      assert_equals(resp.status, 200, "Clean stash response's status is 200");
      return fetch(url + urlParameters, requestInit).then(function(resp) {
        assert_equals(resp.status, 200, "Response's status is 200");
        assert_equals(resp.headers.get("x-did-preflight"), "1", "Preflight request has been made");
        assert_equals(resp.headers.get("x-preflight-referrer"), expectedReferrer, "Preflight's referrer is correct");
        assert_equals(resp.headers.get("x-referrer"), expectedReferrer, "Request's refferer is correct");
      });
    });
  }, desc);
}

var corsUrl = "http://{{host}}:{{ports[http][1]}}" + dirname(location.pathname) + RESOURCES_DIR + "preflight.py";
var origin = "http://{{host}}:{{ports[http][0]}}";

corsPreflightReferrer("Referrer policy: no-referrer", corsUrl, "no-referrer", "");
corsPreflightReferrer("Referrer policy: \"\"", corsUrl, "", "");
corsPreflightReferrer("Referrer policy: origin", corsUrl, "origin", origin);
corsPreflightReferrer("Referrer policy: origin-when-cross-origin", corsUrl, "origin-when-cross-origin", origin);
corsPreflightReferrer("Referrer policy: unsafe-url", corsUrl, "unsafe-url", location.toString());

done();
