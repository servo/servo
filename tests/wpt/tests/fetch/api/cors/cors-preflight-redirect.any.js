// META: global=window,worker
// META: script=/common/utils.js
// META: script=../resources/utils.js
// META: script=/common/get-host-info.sub.js

function corsPreflightRedirect(desc, redirectUrl, redirectLocation, redirectStatus, redirectPreflight) {
  var uuid_token = token();
  var url = redirectUrl;
  var urlParameters = "?token=" + uuid_token + "&max_age=0";
  urlParameters += "&redirect_status=" + redirectStatus;
  urlParameters += "&location=" + encodeURIComponent(redirectLocation);

  if (redirectPreflight)
    urlParameters += "&redirect_preflight";
  var requestInit = {"mode": "cors", "redirect": "follow"};

  /* Force preflight */
  requestInit["headers"] = {"x-force-preflight": ""};
  urlParameters += "&allow_headers=x-force-preflight";

  promise_test(function(test) {
    return fetch(RESOURCES_DIR + "clean-stash.py?token=" + uuid_token).then(function(resp) {
      assert_equals(resp.status, 200, "Clean stash response's status is 200");
      if (redirectPreflight) {
        return promise_rejects_js(test, TypeError, fetch(url + urlParameters, requestInit));
      }
      return fetch(url + urlParameters, requestInit).then(function(resp) {
        assert_equals(resp.status, 200, "Response's status is 200");
        assert_equals(resp.headers.get("x-did-preflight"), "1",
                      "Preflight request has been made for the redirect target");
      });
    });
  }, desc);
}

var redirectUrl = get_host_info().REMOTE_ORIGIN + dirname(location.pathname) + RESOURCES_DIR + "redirect.py";
var locationUrl =  get_host_info().REMOTE_ORIGIN + dirname(location.pathname) + RESOURCES_DIR + "preflight.py";

for (var code of [301, 302, 303, 307, 308]) {
  /* preflight should not follow the redirection */
  corsPreflightRedirect("Redirection " + code + " on preflight failed", redirectUrl, locationUrl, code, true);
  /* preflight is done before redirection: the redirection is followed and preflighted again */
  corsPreflightRedirect("Redirection " + code + " after preflight succeeded", redirectUrl, locationUrl, code, false);
}
