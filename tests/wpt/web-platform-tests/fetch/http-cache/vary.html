<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <title>HTTP Cache - Vary</title>
    <meta name="help" href="https://fetch.spec.whatwg.org/#request">
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
        name: "HTTP cache reuses Vary response when request matches",
        requests: [
          {
            request_headers: [
              ["Foo", "1"]
            ],
            response_headers: [
              ["Expires", 5000],
              ["Last-Modified", -3000],
              ["Vary", "Foo"]
            ]
          },
          {
            request_headers: [
              ["Foo", "1"]
            ],
            expected_type: "cached"
          }
        ]
      },
      {
        name: "HTTP cache doesn't use Vary response when request doesn't match",
        requests: [
          {
            request_headers: [
              ["Foo", "1"]
            ],
            response_headers: [
              ["Expires", 5000],
              ["Last-Modified", -3000],
              ["Vary", "Foo"]
            ]
          },
          {
            request_headers: [
              ["Foo", "2"]
            ],
            expected_type: "not_cached"
          }
        ]
      },
      {
        name: "HTTP cache doesn't use Vary response when request omits variant header",
        requests: [
          {
            request_headers: [
              ["Foo", "1"]
            ],
            response_headers: [
              ["Expires", 5000],
              ["Last-Modified", -3000],
              ["Vary", "Foo"]
            ]
          },
          {
            expected_type: "not_cached"
          }
        ]
      },
      {
        name: "HTTP cache doesn't invalidate existing Vary response",
        requests: [
          {
            request_headers: [
              ["Foo", "1"]
            ],
            response_headers: [
              ["Expires", 5000],
              ["Last-Modified", -3000],
              ["Vary", "Foo"]
            ],
            response_body: http_content('foo_1')
          },
          {
            request_headers: [
              ["Foo", "2"]
            ],
            response_headers: [
              ["Expires", 5000],
              ["Last-Modified", -3000],
              ["Vary", "Foo"]
            ],
            expected_type: "not_cached",
            response_body: http_content('foo_2'),
          },
          {
            request_headers: [
              ["Foo", "1"]
            ],
            response_body: http_content('foo_1'),
            expected_type: "cached"
          }
        ]
      },
      {
        name: "HTTP cache doesn't pay attention to headers not listed in Vary",
        requests: [
          {
            request_headers: [
              ["Foo", "1"],
              ["Other", "2"]
            ],
            response_headers: [
              ["Expires", 5000],
              ["Last-Modified", -3000],
              ["Vary", "Foo"]
            ],
          },
          {
            request_headers: [
              ["Foo", "1"],
              ["Other", "3"]
            ],
            expected_type: "cached"
          }
        ]
      },
      {
        name: "HTTP cache reuses two-way Vary response when request matches",
        requests: [
          {
            request_headers: [
              ["Foo", "1"],
              ["Bar", "abc"]
            ],
            response_headers: [
              ["Expires", 5000],
              ["Last-Modified", -3000],
              ["Vary", "Foo, Bar"]
            ]
          },
          {
            request_headers: [
              ["Foo", "1"],
              ["Bar", "abc"]
            ],
            expected_type: "cached"
          }
        ]
      },
      {
        name: "HTTP cache doesn't use two-way Vary response when request doesn't match",
        requests: [
          {
            request_headers: [
              ["Foo", "1"],
              ["Bar", "abc"]
            ],
            response_headers: [
              ["Expires", 5000],
              ["Last-Modified", -3000],
              ["Vary", "Foo, Bar"]
            ]
          },
          {
            request_headers: [
              ["Foo", "2"],
              ["Bar", "abc"]
            ],
            expected_type: "not_cached"
          }
        ]
      },
      {
        name: "HTTP cache doesn't use two-way Vary response when request omits variant header",
        requests: [
          {
            request_headers: [
              ["Foo", "1"]
            ],
            response_headers: [
              ["Expires", 5000],
              ["Last-Modified", -3000],
              ["Vary", "Foo, Bar"]
            ]
          },
          {
            expected_type: "not_cached"
          }
        ]
      },
      {
        name: "HTTP cache reuses three-way Vary response when request matches",
        requests: [
          {
            request_headers: [
              ["Foo", "1"],
              ["Bar", "abc"],
              ["Baz", "789"]
            ],
            response_headers: [
              ["Expires", 5000],
              ["Last-Modified", -3000],
              ["Vary", "Foo, Bar, Baz"]
            ]
          },
          {
            request_headers: [
              ["Foo", "1"],
              ["Bar", "abc"],
              ["Baz", "789"]
            ],
            expected_type: "cached"
          }
        ]
      },
      {
        name: "HTTP cache doesn't use three-way Vary response when request doesn't match",
        requests: [
          {
            request_headers: [
              ["Foo", "1"],
              ["Bar", "abc"],
              ["Baz", "789"]
            ],
            response_headers: [
              ["Expires", 5000],
              ["Last-Modified", -3000],
              ["Vary", "Foo, Bar, Baz"]
            ]
          },
          {
            request_headers: [
              ["Foo", "2"],
              ["Bar", "abc"],
              ["Baz", "789"]
            ],
            expected_type: "not_cached"
          }
        ]
      },
      {
        name: "HTTP cache doesn't use three-way Vary response when request doesn't match, regardless of header order",
        requests: [
          {
            request_headers: [
              ["Foo", "1"],
              ["Bar", "abc4"],
              ["Baz", "789"]
            ],
            response_headers: [
              ["Expires", 5000],
              ["Last-Modified", -3000],
              ["Vary", "Foo, Bar, Baz"]
            ]
          },
          {
            request_headers: [
              ["Foo", "1"],
              ["Bar", "abc"],
              ["Baz", "789"]
            ],
            expected_type: "not_cached"
          }
        ]
      },
      {
        name: "HTTP cache uses three-way Vary response when both request and the original request omited a variant header",
        requests: [
          {
            request_headers: [
              ["Foo", "1"],
              ["Baz", "789"]
            ],
            response_headers: [
              ["Expires", 5000],
              ["Last-Modified", -3000],
              ["Vary", "Foo, Bar, Baz"]
            ]
          },
          {
            request_headers: [
              ["Foo", "1"],
              ["Baz", "789"]
            ],
            expected_type: "cached"
          }
        ]
      },
      {
        name: "HTTP cache doesn't use Vary response with a field value of '*'",
        requests: [
          {
            request_headers: [
              ["Foo", "1"],
              ["Baz", "789"]
            ],
            response_headers: [
              ["Expires", 5000],
              ["Last-Modified", -3000],
              ["Vary", "*"]
            ]
          },
          {
            request_headers: [
              ["*", "1"],
              ["Baz", "789"]
            ],
            expected_type: "not_cached"
          }
        ]
      }
    ];
    run_tests(tests);
    </script>
  </body>
</html>
