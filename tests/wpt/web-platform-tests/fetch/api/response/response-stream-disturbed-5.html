<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <title>Consuming Response body after getting a ReadableStream</title>
    <meta name="help" href="https://fetch.spec.whatwg.org/#response">
    <meta name="help" href="https://fetch.spec.whatwg.org/#body-mixin">
    <meta name="author" title="Canon Research France" href="https://www.crf.canon.fr">
    <script src="/resources/testharness.js"></script>
    <script src="/resources/testharnessreport.js"></script>
  </head>
  <body>
    <script>

promise_test(function() {
    return fetch("../resources/data.json").then(function(response) {
        response.blob();
        assert_not_equals(response.body, null);
        assert_throws_js(TypeError, function() { response.body.getReader(); });
    });
}, "Getting a body reader after consuming as blob");

promise_test(function() {
    return fetch("../resources/data.json").then(function(response) {
        response.text();
        assert_not_equals(response.body, null);
        assert_throws_js(TypeError, function() { response.body.getReader(); });
    });
}, "Getting a body reader after consuming as text");

promise_test(function() {
    return fetch("../resources/data.json").then(function(response) {
        response.json();
        assert_not_equals(response.body, null);
        assert_throws_js(TypeError, function() { response.body.getReader(); });
    });
}, "Getting a body reader after consuming as json");

promise_test(function() {
    return fetch("../resources/data.json").then(function(response) {
        response.arrayBuffer();
        assert_not_equals(response.body, null);
        assert_throws_js(TypeError, function() { response.body.getReader(); });
    });
}, "Getting a body reader after consuming as arrayBuffer");

    </script>
  </body>
</html>
