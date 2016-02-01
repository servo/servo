if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("../resources/utils.js");
}

function basicAuth(desc, user, pass, mode, status) {
  promise_test(function(test) {
    var headers = { "Authorization": "Basic " + btoa(user + ":" + pass)};
    var requestInit = {"credentials": mode, "headers": headers};
    return fetch(RESOURCES_DIR + "authentication.py?realm=test", requestInit).then(function(resp) {
        assert_equals(resp.status, status, "HTTP status is " + status);
        assert_equals(resp.type , "basic", "Response's type is basic");
    });
  }, desc);
}

basicAuth("User-added Authorization header with include mode", "user", "password", "include", 200);
basicAuth("User-added Authorization header with same-origin mode", "user", "password", "same-origin", 200);
basicAuth("User-added Authorization header with omit mode", "user", "password", "omit", 200);

done();
