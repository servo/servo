<!DOCTYPE html>
<html>
  <head>
    <title>EventSource: request cancellation</title>
    <script src="/resources/testharness.js"></script>
    <script src="/resources/testharnessreport.js"></script>
  </head>
  <body>
    <div id="log"></div>
    <script>
      var t = async_test();
      onload = t.step_func(function() {
        var url = "resources/message.py?sleep=1000&message=" + encodeURIComponent("retry:1000\ndata:abc\n\n");
        var es = new EventSource(url);
        es.onerror = t.step_func(function() {
          assert_equals(es.readyState, EventSource.CLOSED)
          setTimeout(t.step_func(function () {
            assert_equals(es.readyState, EventSource.CLOSED,
                          "After stopping the eventsource readyState should be CLOSED")
            t.done();
          }), 1000);
        });

        setTimeout(t.step_func(function() {
          window.stop()
          es.onopen = t.unreached_func("Got open event");
          es.onmessage = t.unreached_func("Got message after closing source");
        }), 0);
      });
    </script>
  </body>
</html>
