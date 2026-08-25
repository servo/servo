// META: global=window,worker
// META: title=NoVarySearch HTTP Cache
// META: timeout=long
// META: script=/common/utils.js
// META: script=/common/get-host-info.sub.js
// META: script=http-cache.js
/*
NOTE for testing No-Vary-Search-Header:
- If `params` is set to true, `expect=("dispatch" "uuid")` must be specified.
  Otherwise:
  - The same HTTP Cache will be used by other tests, which are supposed
    to be distinguished by uuid.
  - The test utility cannot get the server's states because UA will use the HTTP
    Cache instead of sending a new request to server to ask for the latest state.
- Do not test not_cached cases and cached cases within one test. Test infra
  checks the number of requests and responses without considering if the
  previous responses should be served from cache or not.
*/
var tests = [
  {
    name: "When params is set to true, URL differs only by their parameters (other than `dispatch` and `uuid`) should not be cached as different entries.",
    requests: [
      {
        url_params: "a=1&b=2",
        response_headers: [
          ["Cache-Control", "max-age=10000"],
          ["No-Vary-Search", "params, except=(\"dispatch\" \"uuid\")"],
        ],
      },
      {
        expected_type: "cached"
      }
    ]
  },
  {
    name: "Ground truth: When key-order is not set, URLs should be compared in an order-sensitive way.",
    requests: [
      {
        url_params: "a=1&b=2",
        response_headers: [
          ["Cache-Control", "max-age=10000"],
        ],
      },
      {
        url_params: "b=2&a=1",
        expected_type: "not_cached"
      }
    ]
  },
  {
    name: "When key-order is set , URLs should be compared in an order-insensitive way. Matched cases:",
    requests: [
      {
        url_params: "a=1&b=2",
        response_headers: [
          ["Cache-Control", "max-age=10000"],
          ["No-Vary-Search", "key-order"],
        ],
      },
      {
        url_params: "b=2&a=1",
        expected_type: "cached"
      }
    ]
  },
  {
    name: "When key-order is set , URLs should be compared in an order-insensitive way. Not matched cases",
    requests: [
      {
        url_params: "a=1&b=2",
        response_headers: [
          ["Cache-Control", "max-age=10000"],
          ["No-Vary-Search", "key-order"],
        ],
      },
      {
        url_params: "b=2",
        expected_type: "not_cached"
      },
      {
        url_params: "a=2&b=2",
        expected_type: "not_cached"
      },
      {
        url_params: "a=1&b=2&c=3",
        expected_type: "not_cached"
      }
    ]
  },

  // params as a list of named parameters
  {
    name: "When params names a specific parameter, URLs differing only in that parameter should be cached as the same entry.",
    requests: [
      {
        url_params: "a=1&b=2",
        response_headers: [
          ["Cache-Control", "max-age=10000"],
          ["No-Vary-Search", "params=(\"a\")"],
        ],
      },
      {
        url_params: "a=99&b=2",
        expected_type: "cached"
      }
    ]
  },
  {
    name: "When params names a specific parameter, URLs differing in other parameters should be cached as different entries.",
    requests: [
      {
        url_params: "a=1&b=2",
        response_headers: [
          ["Cache-Control", "max-age=10000"],
          ["No-Vary-Search", "params=(\"a\")"],
        ],
      },
      {
        url_params: "a=1&b=99",
        expected_type: "not_cached"
      }
    ]
  },
  {
    name: "When params names multiple parameters, URLs differing only in those parameters should be cached as the same entry.",
    requests: [
      {
        url_params: "a=1&b=2&c=3",
        response_headers: [
          ["Cache-Control", "max-age=10000"],
          ["No-Vary-Search", "params=(\"a\" \"b\")"],
        ],
      },
      {
        url_params: "a=9&b=8&c=3",
        expected_type: "cached"
      }
    ]
  },

  // params=?1 explicit boolean true
  {
    name: "When params=?1 is set explicitly (equivalent to bare params), URLs differing only in their parameters (other than `dispatch` and `uuid`) should be cached as the same entry.",
    requests: [
      {
        url_params: "a=1&b=2",
        response_headers: [
          ["Cache-Control", "max-age=10000"],
          ["No-Vary-Search", "params=?1, except=(\"dispatch\" \"uuid\")"],
        ],
      },
      {
        expected_type: "cached"
      }
    ]
  },

  // params and except (allowlist)
  {
    name: "When params and except are set, URLs differing in a kept parameter should be cached as different entries.",
    requests: [
      {
        url_params: "id=42&noise=abc",
        response_headers: [
          ["Cache-Control", "max-age=10000"],
          ["No-Vary-Search", "params, except=(\"dispatch\" \"uuid\" \"id\")"],
        ],
      },
      {
        url_params: "id=99&noise=xyz",
        expected_type: "not_cached"
      }
    ]
  },

  // Combined params list and key-order
  {
    name: "When params names a parameter and key-order is set, URLs differing only in that parameter and parameter order should be cached as the same entry.",
    requests: [
      {
        url_params: "utm=x&b=2&a=1",
        response_headers: [
          ["Cache-Control", "max-age=10000"],
          ["No-Vary-Search", "params=(\"utm\"), key-order"],
        ],
      },
      {
        url_params: "a=1&b=2&utm=y",
        expected_type: "cached"
      }
    ]
  },
  {
    name: "When params names a parameter and key-order is set, URLs differing in other parameters should be cached as different entries.",
    requests: [
      {
        url_params: "utm=x&b=2&a=1",
        response_headers: [
          ["Cache-Control", "max-age=10000"],
          ["No-Vary-Search", "params=(\"utm\"), key-order"],
        ],
      },
      {
        url_params: "a=1&b=99&utm=y",
        expected_type: "not_cached"
      }
    ]
  },

  // Invalid NVS values fall back to exact match
  {
    name: "When params is an inner list combined with except, it should fall back to exact match and URLs with different parameters should be cached as different entries.",
    requests: [
      {
        url_params: "a=1",
        response_headers: [
          ["Cache-Control", "max-age=10000"],
          ["No-Vary-Search", "params=(\"a\"), except=(\"b\")"],
        ],
      },
      {
        url_params: "a=2",
        expected_type: "not_cached"
      }
    ]
  },
  {
    name: "When except is set without params, it should fall back to exact match and URLs with different parameters should be cached as different entries.",
    requests: [
      {
        url_params: "a=1",
        response_headers: [
          ["Cache-Control", "max-age=10000"],
          ["No-Vary-Search", "except=(\"a\")"],
        ],
      },
      {
        url_params: "a=2",
        expected_type: "not_cached"
      }
    ]
  }
];
run_tests(tests);
