// META: script=/common/utils.js

promise_test(async () => {
  return fetch("data:text/plain;charset=US-ASCII,paddingHello%2C%20World%21padding", {
    "method": "GET",
    "Range": "bytes=13-26"
  }).then(function(resp) {
    assert_equals(resp.status, 200, "HTTP status is 200");
    assert_equals(resp.type, "basic", "response type is basic");
    assert_equals(resp.headers.get("Content-Type"), "text/plain;charset=US-ASCII", "Content-Type is " + resp.headers.get("Content-Type"));
    return resp.text();
  }).then(function(text) {
    assert_equals(text, 'paddingHello, World!padding', "Response's body ignores range");
  });
}, "data: URL and Range header");

promise_test(async () => {
  return fetch("data:text/plain;charset=US-ASCII,paddingHello%2C%20paddingWorld%21padding", {
    "method": "GET",
    "Range": "bytes=7-14,21-27"
  }).then(function(resp) {
    assert_equals(resp.status, 200, "HTTP status is 200");
    assert_equals(resp.type, "basic", "response type is basic");
    assert_equals(resp.headers.get("Content-Type"), "text/plain;charset=US-ASCII", "Content-Type is " + resp.headers.get("Content-Type"));
    return resp.text();
  }).then(function(text) {
    assert_equals(text, 'paddingHello, paddingWorld!padding', "Response's body ignores range");
  });
}, "data: URL and Range header with multiple ranges");
