<!DOCTYPE html>
<html>
  <head>
    <title>EventSource: Cache-Control</title>
    <script src="/resources/testharness.js"></script>
    <script src="/resources/testharnessreport.js"></script>
  </head>
  <body>
    <div id="log"></div>
    <script>
      var crossdomain = location.href
                    .replace('://', '://www2.')
                    .replace(/\/[^\/]*$/, '/')

      // running it twice to check whether it stays consistent
      function cacheTest(runAgain, url) {
        var test = async_test()
        test.step(function() {
          var source = new EventSource(url)
          source.onmessage = function(e) {
            test.step(function() {
              assert_equals(e.data, "no-cache")
              this.close()
              if(runAgain)
                cacheTest(false, url) // this nests tests
            }, this)
            test.done()
          }
        })
      }

      cacheTest(true, "resources/cache-control.event_stream?pipe=sub")
      cacheTest(true, crossdomain + "resources/cors.py?run=cache-control")
    </script>
  </body>
</html>

