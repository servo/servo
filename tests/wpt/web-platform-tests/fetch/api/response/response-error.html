<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <title>Response error</title>
    <meta name="help" href="https://fetch.spec.whatwg.org/#response">
    <meta name="author" title="Canon Research France" href="https://www.crf.canon.fr">
    <script src="/resources/testharness.js"></script>
    <script src="/resources/testharnessreport.js"></script>
  </head>
  <body>
    <script>
      var invalidStatus = [0, 100, 199, 600, 1000];
      invalidStatus.forEach(function(status) {
        test(function() {
          assert_throws_js(RangeError, function() { new Response("", { "status" : status }); },
            "Expect RangeError exception when status is " + status);
        },"Throws RangeError when responseInit's status is " + status);
      });

      var invalidStatusText = ["\n", "Ā"];
      invalidStatusText.forEach(function(statusText) {
        test(function() {
          assert_throws_js(TypeError, function() { new Response("", { "statusText" : statusText }); },
            "Expect TypeError exception " + statusText);
        },"Throws TypeError when responseInit's statusText is " + statusText);
      });

      var nullBodyStatus = [204, 205, 304];
      nullBodyStatus.forEach(function(status) {
        test(function() {
          assert_throws_js(TypeError,
            function() { new Response("body", {"status" : status }); },
            "Expect TypeError exception ");
        },"Throws TypeError when building a response with body and a body status of " + status);
      });
    </script>
  </body>
</html>
