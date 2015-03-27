<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8">
    <title>EventSource: Last-Event-ID (2)</title>
    <script src="/resources/testharness.js"></script>
    <script src="/resources/testharnessreport.js"></script>
  </head>
  <body>
    <div id="log"></div>
    <script>
      var test = async_test()
      test.step(function() {
        var source = new EventSource("resources/last-event-id.py"),
            counter = 0
        source.onmessage = function(e) {
          test.step(function() {
            if(e.data == "hello" && counter == 0) {
              counter++
              assert_equals(e.lastEventId, "…")
            } else if(counter == 1) {
              counter++
              assert_equals("…", e.data)
              assert_equals("…", e.lastEventId)
            } else if(counter == 2) {
              counter++
              assert_equals("…", e.data)
              assert_equals("…", e.lastEventId)
              source.close()
              test.done()
            } else
              assert_unreached()
          })
        }
      })
    </script>
  </body>
</html>
