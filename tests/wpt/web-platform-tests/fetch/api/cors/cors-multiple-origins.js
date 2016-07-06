if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("../resources/utils.js");
}

function corsMultipleOrigins(desc, originList, shouldPass) {
  var urlParameters = "?origin=" + encodeURIComponent(originList.join(", "));
  var url = "http://{{host}}:{{ports[http][1]}}" + dirname(location.pathname) + RESOURCES_DIR + "preflight.py";

  if (shouldPass) {
    promise_test(function(test) {
      return fetch(url + urlParameters).then(function(resp) {
        assert_equals(resp.status, 200, "Response's status is 200");
      });
    }, desc);
  } else {
    promise_test(function(test) {
      return promise_rejects(test, new TypeError(), fetch(url + urlParameters));
    }, desc);
  }
}
/* Actual origin */
var origin = "http://{{host}}:{{ports[http][0]}}";

corsMultipleOrigins("3 origins allowed, match the 3rd (" + origin + ")", ["\"\"", "http://example.com", origin], true);
corsMultipleOrigins("3 origins allowed, match the 3rd (\"*\")", ["\"\"", "http://example.com", "*"], true);
corsMultipleOrigins("3 origins allowed, match twice (" + origin + ")", ["\"\"", origin, origin], true);
corsMultipleOrigins("3 origins allowed, match twice (\"*\")", ["*", "http://example.com", "*"], true);
corsMultipleOrigins("3 origins allowed, match twice (\"*\" and " + origin + ")", ["*", "http://example.com", origin], true);
corsMultipleOrigins("3 origins allowed, no match", ["", "http://example.com", "https://example2.com"], false);

done();
