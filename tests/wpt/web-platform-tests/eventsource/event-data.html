<!doctype html>
<html>
  <head>
    <title>EventSource: lines and data parsing</title>
    <meta rel=help href="http://dev.w3.org/html5/eventsource/#event-stream-interpretation">
  <meta rel=assert title="If the line is empty (a blank line) Dispatch the event, as defined below.">
  <meta rel=assert title="If the line starts with a U+003A COLON character (:) Ignore the line.">
  <meta rel=assert title="If the line contains a U+003A COLON character (:)
  Collect the characters on the line before the first U+003A COLON character (:), and let field be that string.
  Collect the characters on the line after the first U+003A COLON character (:), and let value be that string. If value starts with a U+0020 SPACE character, remove it from value.
  Process the field using the steps described below, using field as the field name and value as the field value.
  ">
  <meta rel=assert title="Otherwise, the string is not empty but does not contain a U+003A COLON character (:)
Process the field using the steps described below, using the whole line as the field name, and the empty string as the field value.
  ">

    <script src="/resources/testharness.js"></script>
    <script src="/resources/testharnessreport.js"></script>
  </head>
  <body>
    <div id="log"></div>
    <script>
      var test = async_test();
      test.step(function() {
        var source = new EventSource("resources/message2.py"),
            counter = 0;
        source.onmessage = test.step_func(function(e) {
          if(counter == 0) {
            assert_equals(e.data,"msg\nmsg");
          } else if(counter == 1) {
            assert_equals(e.data,"");
          } else if(counter == 2) {
            assert_equals(e.data,"end");
            source.close();
            test.done();
          } else {
            assert_unreached();
          }
          counter++;
        });
      });
    </script>
  </body>
</html>
