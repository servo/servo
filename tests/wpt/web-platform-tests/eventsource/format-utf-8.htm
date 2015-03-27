<!DOCTYPE html>
<html>
  <head>
    <meta charset=utf-8>
    <title>EventSource: always UTF-8</title>
    <script src="/resources/testharness.js"></script>
    <script src="/resources/testharnessreport.js"></script>
  </head>
  <body>
    <div id="log"></div>
    <script>
      async_test().step(function() {
        var source = new EventSource("resources/message.py?mime=text/event-stream;charset=windows-1252&message=data%3Aok%E2%80%A6")
        source.onmessage = this.step_func(function(e) {
          assert_equals('ok…', e.data, 'decoded data')
          source.close()
          this.done()
        })
        source.onerror = this.step_func(function() {
          assert_unreached("Got error event")
        })
      })
    </script>
  </body>
</html>

