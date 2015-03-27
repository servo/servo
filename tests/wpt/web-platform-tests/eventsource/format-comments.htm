<!doctype html>
<html>
  <head>
    <title>EventSource: comment fest</title>
    <script src="/resources/testharness.js"></script>
    <script src="/resources/testharnessreport.js"></script>
  </head>
  <body>
    <div id="log"></div>
    <script>
      var test = async_test()
      test.step(function() {
        var longstring = (new Array(2*1024+1)).join("x"), // cannot make the string too long; causes timeout
            message = encodeURI("data:1\r:\0\n:\r\ndata:2\n:" + longstring + "\rdata:3\n:data:fail\r:" + longstring + "\ndata:4\n"),
            source = new EventSource("resources/message.py?message=" + message + "&newline=none")
        source.onmessage = function(e) {
          test.step(function() {
            assert_equals("1\n2\n3\n4", e.data)
            source.close()
          })
          test.done()
        }
      })
    </script>
  </body>
</html>

