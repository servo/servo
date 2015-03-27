<!doctype html>
<html>
  <head>
    <title>EventSource: unknown fields and parsing fun</title>
    <script src="/resources/testharness.js"></script>
    <script src="/resources/testharnessreport.js"></script>
  </head>
  <body>
    <div id="log"></div>
    <script>
      var test = async_test()
      test.step(function() {
        var source = new EventSource("resources/message.py?message=data%3Atest%0A%20data%0Adata%0Afoobar%3Axxx%0Ajustsometext%0A%3Athisisacommentyay%0Adata%3Atest")
        source.onmessage = function(e) {
          test.step(function() {
            assert_equals("test\n\ntest", e.data)
            source.close()
          })
          test.done()
        }
      })
    </script>
  </body>
</html>

