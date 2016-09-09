if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("../resources/utils.js");
  importScripts("/common/get-host-info.sub.js")
}

function fetchSameOrigin(url, shouldPass) {
  promise_test(function(test) {
    if (shouldPass)
      return fetch(url , {"mode": "same-origin"}).then(function(resp) {
        assert_equals(resp.status, 200, "HTTP status is 200");
        assert_equals(resp.type, "basic", "response type is basic");
      });
    else
      return promise_rejects(test, new TypeError, fetch(url, {mode: "same-origin"}));
  }, "Fetch "+ url + " with same-origin mode");
}

var host_info = get_host_info();

fetchSameOrigin(RESOURCES_DIR + "top.txt", true);
fetchSameOrigin(host_info.HTTP_ORIGIN + "/fetch/api/resources/top.txt", true);
fetchSameOrigin(host_info.HTTPS_ORIGIN + "/fetch/api/resources/top.txt", false);
fetchSameOrigin(host_info.HTTP_REMOTE_ORIGIN + "/fetch/api/resources/top.txt", false);

var redirPath = dirname(location.pathname) + RESOURCES_DIR + "redirect.py?location=";

fetchSameOrigin(redirPath + RESOURCES_DIR + "top.txt", true);
fetchSameOrigin(redirPath + host_info.HTTP_ORIGIN + "/fetch/api/resources/top.txt", true);
fetchSameOrigin(redirPath + host_info.HTTPS_ORIGIN + "/fetch/api/resources/top.txt", false);
fetchSameOrigin(redirPath + host_info.HTTP_REMOTE_ORIGIN + "/fetch/api/resources/top.txt", false);

done();

