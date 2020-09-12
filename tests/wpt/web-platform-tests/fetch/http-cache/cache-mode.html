<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <title>Fetch - Cache Mode</title>
    <meta name="help" href="https://fetch.spec.whatwg.org/#http-network-or-cache-fetch">
    <meta name="timeout" content="long">
    <script src="/resources/testharness.js"></script>
    <script src="/resources/testharnessreport.js"></script>
    <script src="/common/utils.js"></script>
    <script src="/common/get-host-info.sub.js"></script>
    <script src="http-cache.js"></script>
  </head>
  <body>
    <script>
    var tests = [
      {
        name: "Fetch sends Cache-Control: max-age=0 when cache mode is no-cache",
        requests: [
          {
            cache: "no-cache",
            expected_request_headers: [['cache-control', 'max-age=0']]
          }
        ]
      },
      {
        name: "Fetch doesn't touch Cache-Control when cache mode is no-cache and Cache-Control is already present",
        requests: [
          {
            cache: "no-cache",
            request_headers: [['cache-control', 'foo']],
            expected_request_headers: [['cache-control', 'foo']]
          }
        ]
      },
      {
        name: "Fetch sends Cache-Control: no-cache and Pragma: no-cache when cache mode is no-store",
        requests: [
          {
            cache: "no-store",
            expected_request_headers: [
              ['cache-control', 'no-cache'],
              ['pragma', 'no-cache']
            ]
          }
        ]
      },
      {
        name: "Fetch doesn't touch Cache-Control when cache mode is no-store and Cache-Control is already present",
        requests: [
          {
            cache: "no-store",
            request_headers: [['cache-control', 'foo']],
            expected_request_headers: [['cache-control', 'foo']]
          }
        ]
      },
      {
        name: "Fetch doesn't touch Pragma when cache mode is no-store and Pragma is already present",
        requests: [
          {
            cache: "no-store",
            request_headers: [['pragma', 'foo']],
            expected_request_headers: [['pragma', 'foo']]
          }
        ]
      }
    ];
    run_tests(tests);
    </script>
  </body>
</html>
