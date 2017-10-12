<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <title>Headers have combined (and sorted) values</title>
    <meta name="help" href="https://fetch.spec.whatwg.org/#headers">
    <meta name="author" title="Canon Research France" href="https://www.crf.canon.fr">
    <script src="/resources/testharness.js"></script>
    <script src="/resources/testharnessreport.js"></script>
  </head>
  <body>
    <script>
      var headerSeqCombine = [["single", "singleValue"],
                              ["double", "doubleValue1"],
                              ["double", "doubleValue2"],
                              ["triple", "tripleValue1"],
                              ["triple", "tripleValue2"],
                              ["triple", "tripleValue3"]
      ];
      var expectedDict = {"single": "singleValue",
                          "double": "doubleValue1, doubleValue2",
                          "triple": "tripleValue1, tripleValue2, tripleValue3"
      };

      test(function() {
        var headers = new Headers(headerSeqCombine);
        for (name in expectedDict)
          assert_equals(headers.get(name), expectedDict[name]);
      }, "Create headers using same name for different values");

      test(function() {
        var headers = new Headers(headerSeqCombine);
        for (name in expectedDict) {
          assert_true(headers.has(name), "name: " + name + " has value(s)");
          headers.delete(name);
          assert_false(headers.has(name), "name: " + name + " has no value(s) anymore");
        }
      }, "Check delete and has methods when using same name for different values");

      test(function() {
        var headers = new Headers(headerSeqCombine);
        for (name in expectedDict) {
          headers.set(name,"newSingleValue");
          assert_equals(headers.get(name), "newSingleValue", "name: " + name + " has value: newSingleValue");
        }
      }, "Check set methods when called with already used name");

      test(function() {
        var headers = new Headers(headerSeqCombine);
        for (name in expectedDict) {
          var value = headers.get(name);
          headers.append(name,"newSingleValue");
          assert_equals(headers.get(name), (value + ", " + "newSingleValue"));
        }
      }, "Check append methods when called with already used name");

      test(() => {
        const headers = new Headers([["1", "a"],["1", "b"]]);
        for(let header of headers) {
          assert_array_equals(header, ["1", "a, b"]);
        }
      }, "Iterate combined values");

      test(() => {
        const headers = new Headers([["2", "a"], ["1", "b"], ["2", "b"]]),
              expected = [["1", "b"], ["2", "a, b"]];
        let i = 0;
        for(let header of headers) {
          assert_array_equals(header, expected[i]);
          i++;
        }
        assert_equals(i, 2);
      }, "Iterate combined values in sorted order")
    </script>
  </body>
</html>
