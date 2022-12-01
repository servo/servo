// META: global=window,worker
// META: title=Response: error static method

test(function() {
  var responseError = Response.error();
  assert_equals(responseError.type, "error", "Network error response's type is error");
  assert_equals(responseError.status, 0, "Network error response's status is 0");
  assert_equals(responseError.statusText, "", "Network error response's statusText is empty");
  assert_equals(responseError.body, null, "Network error response's body is null");

  assert_true(responseError.headers.entries().next().done, "Headers should be empty");
}, "Check response returned by static method error()");

promise_test (async function() {
  let response = await fetch("../resources/data.json");

  try {
    response.headers.append('name', 'value');
  } catch (e) {
    assert_equals(e.constructor.name, "TypeError");
  }

  assert_not_equals(response.headers.get("name"), "value", "response headers should be immutable");
}, "Ensure response headers are immutable");

test(function() {
  const headers = Response.error().headers;

  // Avoid false positives if expected API is not available
  assert_true(!!headers);
  assert_equals(typeof headers.append, 'function');

  assert_throws_js(TypeError, function () { headers.append('name', 'value'); });
}, "the 'guard' of the Headers instance should be immutable");
