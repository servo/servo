<!DOCTYPE html>
<html>
  <head>
    <meta charset=utf-8>
    <title>EventSource: null character in response</title>
    <script src="/resources/testharness.js"></script>
    <script src="/resources/testharnessreport.js"></script>
  </head>
  <body>
    <div id="log"></div>
    <script>
      var test = async_test()
      test.step(function() {
        var source = new EventSource("resources/message.py?message=data%3A%00%0A%0A")
        source.onmessage = function(e) {
          test.step(function() {
            assert_equals("\x00", e.data)
            source.close()
          }, this)
          test.done()
        }
        source.onerror = function() {
          test.step(function() { assert_unreached() })
          test.done()
        }
      })
    </script>
  </body>
</html>

