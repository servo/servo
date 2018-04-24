// META: script=../resources/utils.js

function checkNetworkError(url, method) {
  method = method || "GET";
  const desc = "Fetching " + url.substring(0, 45) + " with method " + method + " is KO"
  promise_test(function(test) {
    var promise = fetch(url, { method: method });
    return promise_rejects(test, new TypeError(), promise);
  }, desc);
}

checkNetworkError("about:blank", "GET");
checkNetworkError("about:blank", "PUT");
checkNetworkError("about:blank", "POST");
checkNetworkError("about:invalid.com");
checkNetworkError("about:config");
checkNetworkError("about:unicorn");
