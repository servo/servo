if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("../resources/utils.js");
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

fetchSameOrigin(RESOURCES_DIR + "top.txt", true);
fetchSameOrigin("http://{{host}}:{{ports[http][0]}}/fetch/api/resources/top.txt", true);
fetchSameOrigin("https://{{host}}:{{ports[https][0]}}/fetch/api/resources/top.txt", false);
fetchSameOrigin("http://{{domains[www]}}:{{ports[http][0]}}/fetch/api/resources/top.txt", false);

done();

