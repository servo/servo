// META: global=window,worker
// META: title=NoVarySearch HTTP Cache
// META: timeout=long
// META: script=/common/utils.js
// META: script=/common/get-host-info.sub.js
// META: script=http-cache.js
/*
NOTE for testing No-Vary-Search-Header:
If `params` is set to true, `expect=("dispatch" "uuid")` must be specified.
Otherwise:
- The same HTTP Cache will be used by other tests, which are supposed
  to be distinguished by uuid.
- The test utility cannot get the server's states because UA will use the HTTP
  Cache instead of sending a new request to server to ask for the latest state.
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
  }
];
run_tests(tests);
