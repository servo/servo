<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <title>HTTP Cache - Caching POST and PATCH responses</title>
    <meta name="help" href="https://fetch.spec.whatwg.org/#request">
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
        name: "HTTP cache uses content after PATCH request with response containing Content-Location and cache-allowing header",
        requests: [
          {
            request_method: "PATCH",
            request_body: "abc",
            response_status: [200, "OK"],
            response_headers: [
              ['Cache-Control', "private, max-age=1000"],
              ['Content-Location', ""]
            ],
            response_body: "abc"
          },
          {
            expected_type: "cached"
          }
        ]
      },
      {
        name: "HTTP cache uses content after POST request with response containing Content-Location and cache-allowing header",
        requests: [
          {
            request_method: "POST",
            request_body: "abc",
            response_status: [200, "OK"],
            response_headers: [
              ['Cache-Control', "private, max-age=1000"],
              ['Content-Location', ""]
            ],
            response_body: "abc"
          },
          {
            expected_type: "cached"
          }
        ]
      }
    ];
    run_tests(tests);
    </script>
  </body>
</html>
