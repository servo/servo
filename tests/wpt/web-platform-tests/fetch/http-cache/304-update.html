<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <title>HTTP Cache - 304 Updates</title>
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
        name: "HTTP cache updates returned headers from a Last-Modified 304",
        requests: [
          {
            response_headers: [
              ["Expires", -5000],
              ["Last-Modified", -3000],
              ["Test-Header", "A"]
            ]
          },
          {
            response_headers: [
              ["Expires", -3000],
              ["Last-Modified", -3000],
              ["Test-Header", "B"]
            ],
            expected_type: "lm_validated",
            expected_response_headers: [
              ["Test-Header", "B"]
            ]
          }
        ]
      },
      {
        name: "HTTP cache updates stored headers from a Last-Modified 304",
        requests: [
          {
            response_headers: [
              ["Expires", -5000],
              ["Last-Modified", -3000],
              ["Test-Header", "A"]
            ]
          },
          {
            response_headers: [
              ["Expires", 3000],
              ["Last-Modified", -3000],
              ["Test-Header", "B"]
            ],
            expected_type: "lm_validated",
            expected_response_headers: [
              ["Test-Header", "B"]
            ],
            pause_after: true
          },
          {
            expected_type: "cached",
            expected_response_headers: [
              ["Test-Header", "B"]
            ]
          }
        ]
      },
      {
        name: "HTTP cache updates returned headers from a ETag 304",
        requests: [
          {
            response_headers: [
              ["Expires", -5000],
              ["ETag", "ABC"],
              ["Test-Header", "A"]
            ]
          },
          {
            response_headers: [
              ["Expires", -3000],
              ["ETag", "ABC"],
              ["Test-Header", "B"]
            ],
            expected_type: "etag_validated",
            expected_response_headers: [
              ["Test-Header", "B"]
            ]
          }
        ]
      },
      {
        name: "HTTP cache updates stored headers from a ETag 304",
        requests: [
          {
            response_headers: [
              ["Expires", -5000],
              ["ETag", "DEF"],
              ["Test-Header", "A"]
            ]
          },
          {
            response_headers: [
              ["Expires", 3000],
              ["ETag", "DEF"],
              ["Test-Header", "B"]
            ],
            expected_type: "etag_validated",
            expected_response_headers: [
              ["Test-Header", "B"]
            ],
            pause_after: true
          },
          {
            expected_type: "cached",
            expected_response_headers: [
              ["Test-Header", "B"]
            ]
          }
        ]
      },
      {
        name: "Content-* header",
        requests: [
          {
            response_headers: [
              ["Expires", -5000],
              ["ETag", "GHI"],
              ["Content-Test-Header", "A"]
            ]
          },
          {
            response_headers: [
              ["Expires", 3000],
              ["ETag", "GHI"],
              ["Content-Test-Header", "B"]
            ],
            expected_type: "etag_validated",
            expected_response_headers: [
              ["Content-Test-Header", "B"]
            ],
            pause_after: true
          },
          {
            expected_type: "cached",
            expected_response_headers: [
              ["Content-Test-Header", "B"]
            ]
          }
        ]
      },
    ];
    run_tests(tests);
    </script>
  </body>
</html>
