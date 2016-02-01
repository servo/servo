<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <title>Headers case management</title>
    <meta name="help" href="https://fetch.spec.whatwg.org/#concept-header-list-append">
    <meta name="author" title="Canon Research France" href="https://www.crf.canon.fr">
    <script src="/resources/testharness.js"></script>
    <script src="/resources/testharnessreport.js"></script>
  </head>
  <body>
    <script>
      var headerDictCase = {"UPPERCASE": "value1",
                            "lowercase": "value2",
                            "mixedCase": "value3",
                            "Content-TYPE": "value4"
                           };

      function checkHeadersCase(originalName, headersToCheck, expectedDict) {
        var lowCaseName = originalName.toLowerCase();
        var upCaseName = originalName.toUpperCase();
        var expectedValue = expectedDict[originalName];
        assert_equals(headersToCheck.get(originalName), expectedValue,
            "name: " + originalName + " has value: " + expectedValue);
        assert_equals(headersToCheck.get(lowCaseName), expectedValue,
            "name: " + lowCaseName + " has value: " + expectedValue);
        assert_equals(headersToCheck.get(upCaseName), expectedValue,
            "name: " + upCaseName + " has value: " + expectedValue);
      }

      test(function() {
        var headers = new Headers(headerDictCase);
        for (name in headerDictCase)
          checkHeadersCase(name, headers, headerDictCase)
      }, "Create headers, names use characters with different case");

      test(function() {
        var headers = new Headers();
        for (name in headerDictCase) {
          headers.append(name, headerDictCase[name]);
          checkHeadersCase(name, headers, headerDictCase);
        }
      }, "Check append method, names use characters with different case");

      test(function() {
        var headers = new Headers();
        for (name in headerDictCase) {
          headers.set(name, headerDictCase[name]);
          checkHeadersCase(name, headers, headerDictCase);
        }
      }, "Check set method, names use characters with different case");

      test(function() {
        var headers = new Headers();
        for (name in headerDictCase)
          headers.set(name, headerDictCase[name]);
        for (name in headerDictCase)
          headers.delete(name.toLowerCase());
        for (name in headerDictCase)
          assert_false(headers.has(name), "header " + name + " should have been deleted");
      }, "Check delete method, names use characters with different case");
    </script>
  </body>
</html>
