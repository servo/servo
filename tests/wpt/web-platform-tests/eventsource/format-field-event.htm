<!doctype html>
<html>
  <head>
    <title>EventSource: custom event name</title>
    <script src="/resources/testharness.js"></script>
    <script src="/resources/testharnessreport.js"></script>
  </head>
  <body>
    <div id="log"></div>
    <script>
      var test = async_test(),
          dispatchedtest = false
      test.step(function() {
        var source = new EventSource("resources/message.py?message=event%3Atest%0Adata%3Ax%0A%0Adata%3Ax")
        source.addEventListener("test", function() { test.step(function() { dispatchedtest = true }) }, false)
        source.onmessage = function() {
          test.step(function() {
            assert_true(dispatchedtest)
            this.close()
          }, this)
          test.done()
        }
      })
    </script>
  </body>
</html>

