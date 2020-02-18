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

function createResponseWithLockedReadableStream(callback) {
    return fetch("../resources/data.json").then(function(response) {
        var reader = response.body.getReader();
        return callback(response);
    });
}

promise_test(function(test) {
    return createResponseWithLockedReadableStream(function(response) {
        return promise_rejects_js(test, TypeError, response.blob());
    });
}, "Getting blob after getting a locked Response body");

promise_test(function(test) {
    return createResponseWithLockedReadableStream(function(response) {
        return promise_rejects_js(test, TypeError, response.text());
    });
}, "Getting text after getting a locked Response body");

promise_test(function(test) {
    return createResponseWithLockedReadableStream(function(response) {
        return promise_rejects_js(test, TypeError, response.json());
    });
}, "Getting json after getting a locked Response body");

promise_test(function(test) {
    return createResponseWithLockedReadableStream(function(response) {
        return promise_rejects_js(test, TypeError, response.arrayBuffer());
    });
}, "Getting arrayBuffer after getting a locked Response body");

    </script>
  </body>
</html>
