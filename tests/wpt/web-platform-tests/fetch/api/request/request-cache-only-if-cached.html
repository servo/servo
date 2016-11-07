<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <title>Request cache - only-if-cached</title>
    <meta name="help" href="https://fetch.spec.whatwg.org/#request">
    <script src="/resources/testharness.js"></script>
    <script src="/resources/testharnessreport.js"></script>
    <script src="/common/utils.js"></script>
    <script src="/common/get-host-info.sub.js"></script>
    <script src="request-cache.js"></script>
  </head>
  <body>
    <script>
    var tests = [
      {
        name: 'RequestCache "only-if-cached" mode checks the cache for previously cached content and avoids revalidation for stale responses',
        state: "stale",
        request_cache: ["default", "only-if-cached"],
        expected_validation_headers: [false],
        expected_no_cache_headers: [false]
      },
      {
        name: 'RequestCache "only-if-cached" mode checks the cache for previously cached content and avoids revalidation for fresh responses',
        state: "fresh",
        request_cache: ["default", "only-if-cached"],
        expected_validation_headers: [false],
        expected_no_cache_headers: [false]
      },
      {
        name: 'RequestCache "only-if-cached" mode checks the cache for previously cached content and does not go to the network if a cached response is not found',
        state: "fresh",
        request_cache: ["only-if-cached"],
        response: ["error"],
        expected_validation_headers: [],
        expected_no_cache_headers: []
      },
      {
        name: 'RequestCache "only-if-cached" (with "same-origin") uses cached same-origin redirects to same-origin content',
        state: "fresh",
        request_cache: ["default", "only-if-cached"],
        redirect: "same-origin",
        expected_validation_headers: [false, false],
        expected_no_cache_headers: [false, false],
      },
      {
        name: 'RequestCache "only-if-cached" (with "same-origin") uses cached same-origin redirects to same-origin content',
        state: "stale",
        request_cache: ["default", "only-if-cached"],
        redirect: "same-origin",
        expected_validation_headers: [false, false],
        expected_no_cache_headers: [false, false],
      },
      {
        name: 'RequestCache "only-if-cached" (with "same-origin") does not follow redirects across origins and rejects',
        state: "fresh",
        request_cache: ["default", "only-if-cached"],
        redirect: "cross-origin",
        response: [null, "error"],
        expected_validation_headers: [false, false],
        expected_no_cache_headers: [false, false],
      },
      {
        name: 'RequestCache "only-if-cached" (with "same-origin") does not follow redirects across origins and rejects',
        state: "stale",
        request_cache: ["default", "only-if-cached"],
        redirect: "cross-origin",
        response: [null, "error"],
        expected_validation_headers: [false, false],
        expected_no_cache_headers: [false, false],
      },
    ];
    run_tests(tests);
    </script>
  </body>
</html>
