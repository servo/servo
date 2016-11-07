<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <title>Request cache - force-cache</title>
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
        name: 'RequestCache "force-cache" mode checks the cache for previously cached content and avoid revalidation for stale responses',
        state: "stale",
        request_cache: ["default", "force-cache"],
        expected_validation_headers: [false],
        expected_no_cache_headers: [false],
      },
      {
        name: 'RequestCache "force-cache" mode checks the cache for previously cached content and avoid revalidation for fresh responses',
        state: "fresh",
        request_cache: ["default", "force-cache"],
        expected_validation_headers: [false],
        expected_no_cache_headers: [false],
      },
      {
        name: 'RequestCache "force-cache" mode checks the cache for previously cached content and goes to the network if a cached response is not found',
        state: "stale",
        request_cache: ["force-cache"],
        expected_validation_headers: [false],
        expected_no_cache_headers: [false],
      },
      {
        name: 'RequestCache "force-cache" mode checks the cache for previously cached content and goes to the network if a cached response is not found',
        state: "fresh",
        request_cache: ["force-cache"],
        expected_validation_headers: [false],
        expected_no_cache_headers: [false],
      },
      {
        name: 'RequestCache "force-cache" mode checks the cache for previously cached content and goes to the network if a cached response would vary',
        state: "stale",
        vary: "*",
        request_cache: ["default", "force-cache"],
        expected_validation_headers: [false, true],
        expected_no_cache_headers: [false, false],
      },
      {
        name: 'RequestCache "force-cache" mode checks the cache for previously cached content and goes to the network if a cached response would vary',
        state: "fresh",
        vary: "*",
        request_cache: ["default", "force-cache"],
        expected_validation_headers: [false, true],
        expected_no_cache_headers: [false, false],
      },
      {
        name: 'RequestCache "force-cache" stores the response in the cache if it goes to the network',
        state: "stale",
        request_cache: ["force-cache", "default"],
        expected_validation_headers: [false, true],
        expected_no_cache_headers: [false, false],
      },
      {
        name: 'RequestCache "force-cache" stores the response in the cache if it goes to the network',
        state: "fresh",
        request_cache: ["force-cache", "default"],
        expected_validation_headers: [false],
        expected_no_cache_headers: [false],
      },
    ];
    run_tests(tests);
    </script>
  </body>
</html>
