// META: script=/common/subset-tests-by-key.js
// META: script=/common/utils.js
// META: script=resources/support.sub.js
// META: variant=?include=baseline
// META: variant=?include=from-local
// META: variant=?include=from-private
// META: variant=?include=from-public
//
// Spec: https://wicg.github.io/private-network-access/#integration-fetch
//
// These tests verify that secure contexts can fetch subresources from all
// address spaces, provided that the target server, if more private than the
// initiator, respond affirmatively to preflight requests.
//
// This file covers only those tests that must execute in a secure context.
// Other tests are defined in: fetch.window.js

setup(() => {
  // Making sure we are in a secure context, as expected.
  assert_true(window.isSecureContext);
});

// Source: secure local context.
//
// All fetches unaffected by Private Network Access.

subsetTestByKey("from-local", promise_test, t => fetchTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: { server: Server.HTTPS_LOCAL },
  expected: FetchTestResult.SUCCESS,
}), "local to local: no preflight required.");

subsetTestByKey("from-local", promise_test, t => fetchTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: FetchTestResult.SUCCESS,
}), "local to private: no preflight required.");


subsetTestByKey("from-local", promise_test, t => fetchTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: {
    server: Server.HTTPS_PUBLIC,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: FetchTestResult.SUCCESS,
}), "local to public: no preflight required.");

// Strictly speaking, the following two tests do not exercise PNA-specific
// logic, but they serve as a baseline for comparison, ensuring that non-PNA
// preflight requests are sent and handled as expected.

subsetTestByKey("baseline", promise_test, t => fetchTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: {
    server: Server.HTTPS_PUBLIC,
    behavior: {
      preflight: PreflightBehavior.failure(),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  fetchOptions: { method: "PUT" },
  expected: FetchTestResult.FAILURE,
}), "local to public: PUT preflight failure.");

subsetTestByKey("baseline", promise_test, t => fetchTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: {
    server: Server.HTTPS_PUBLIC,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    }
  },
  fetchOptions: { method: "PUT" },
  expected: FetchTestResult.SUCCESS,
}), "local to public: PUT preflight success.");

// Generates tests of preflight behavior for a single (source, target) pair.
//
// Scenarios:
//
//  - cors mode:
//    - preflight response has non-2xx HTTP code
//    - preflight response is missing CORS headers
//    - preflight response is missing the PNA-specific `Access-Control` header
//    - final response is missing CORS headers
//    - success
//    - success with PUT method (non-"simple" request)
//  - no-cors mode:
//    - preflight response has non-2xx HTTP code
//    - preflight response is missing CORS headers
//    - preflight response is missing the PNA-specific `Access-Control` header
//    - success
//
function makePreflightTests({
  subsetKey,
  source,
  sourceDescription,
  targetServer,
  targetDescription,
}) {
  const prefix =
      `${sourceDescription} to ${targetDescription}: `;

  subsetTestByKey(subsetKey, promise_test, t => fetchTest(t, {
    source,
    target: {
      server: targetServer,
      behavior: {
        preflight: PreflightBehavior.failure(),
        response: ResponseBehavior.allowCrossOrigin(),
      },
    },
    expected: FetchTestResult.FAILURE,
  }), prefix + "failed preflight.");

  subsetTestByKey(subsetKey, promise_test, t => fetchTest(t, {
    source,
    target: {
      server: targetServer,
      behavior: {
        preflight: PreflightBehavior.noCorsHeader(token()),
        response: ResponseBehavior.allowCrossOrigin(),
      },
    },
    expected: FetchTestResult.FAILURE,
  }), prefix + "missing CORS headers on preflight response.");

  subsetTestByKey(subsetKey, promise_test, t => fetchTest(t, {
    source,
    target: {
      server: targetServer,
      behavior: {
        preflight: PreflightBehavior.noPnaHeader(token()),
        response: ResponseBehavior.allowCrossOrigin(),
      },
    },
    expected: FetchTestResult.FAILURE,
  }), prefix + "missing PNA header on preflight response.");

  subsetTestByKey(subsetKey, promise_test, t => fetchTest(t, {
    source,
    target: {
      server: targetServer,
      behavior: { preflight: PreflightBehavior.success(token()) },
    },
    expected: FetchTestResult.FAILURE,
  }), prefix + "missing CORS headers on final response.");

  subsetTestByKey(subsetKey, promise_test, t => fetchTest(t, {
    source,
    target: {
      server: targetServer,
      behavior: {
        preflight: PreflightBehavior.success(token()),
        response: ResponseBehavior.allowCrossOrigin(),
      },
    },
    expected: FetchTestResult.SUCCESS,
  }), prefix + "success.");

  subsetTestByKey(subsetKey, promise_test, t => fetchTest(t, {
    source,
    target: {
      server: targetServer,
      behavior: {
        preflight: PreflightBehavior.success(token()),
        response: ResponseBehavior.allowCrossOrigin(),
      },
    },
    fetchOptions: { method: "PUT" },
    expected: FetchTestResult.SUCCESS,
  }), prefix + "PUT success.");

  subsetTestByKey(subsetKey, promise_test, t => fetchTest(t, {
    source,
    target: { server: targetServer },
    fetchOptions: { mode: "no-cors" },
    expected: FetchTestResult.FAILURE,
  }), prefix + "no-CORS mode failed preflight.");

  subsetTestByKey(subsetKey, promise_test, t => fetchTest(t, {
    source,
    target: {
      server: targetServer,
      behavior: { preflight: PreflightBehavior.noCorsHeader(token()) },
    },
    fetchOptions: { mode: "no-cors" },
    expected: FetchTestResult.FAILURE,
  }), prefix + "no-CORS mode missing CORS headers on preflight response.");

  subsetTestByKey(subsetKey, promise_test, t => fetchTest(t, {
    source,
    target: {
      server: targetServer,
      behavior: { preflight: PreflightBehavior.noPnaHeader(token()) },
    },
    fetchOptions: { mode: "no-cors" },
    expected: FetchTestResult.FAILURE,
  }), prefix + "no-CORS mode missing PNA header on preflight response.");

  subsetTestByKey(subsetKey, promise_test, t => fetchTest(t, {
    source,
    target: {
      server: targetServer,
      behavior: { preflight: PreflightBehavior.success(token()) },
    },
    fetchOptions: { mode: "no-cors" },
    expected: FetchTestResult.OPAQUE,
  }), prefix + "no-CORS mode success.");
}

// Source: private secure context.
//
// Fetches to the local address space require a successful preflight response
// carrying a PNA-specific header.

makePreflightTests({
  subsetKey: "from-private",
  source: { server: Server.HTTPS_PRIVATE },
  sourceDescription: "private",
  targetServer: Server.HTTPS_LOCAL,
  targetDescription: "local",
});

subsetTestByKey("from-private", promise_test, t => fetchTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: { server: Server.HTTPS_PRIVATE },
  expected: FetchTestResult.SUCCESS,
}), "private to private: no preflight required.");

subsetTestByKey("from-private", promise_test, t => fetchTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: {
    server: Server.HTTPS_PUBLIC,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: FetchTestResult.SUCCESS,
}), "private to public: no preflight required.");

// Source: public secure context.
//
// Fetches to the local and private address spaces require a successful
// preflight response carrying a PNA-specific header.

makePreflightTests({
  subsetKey: "from-public",
  source: { server: Server.HTTPS_PUBLIC },
  sourceDescription: "public",
  targetServer: Server.HTTPS_LOCAL,
  targetDescription: "local",
});

makePreflightTests({
  subsetKey: "from-public",
  source: { server: Server.HTTPS_PUBLIC },
  sourceDescription: "public",
  targetServer: Server.HTTPS_PRIVATE,
  targetDescription: "private",
});

subsetTestByKey("from-public", promise_test, t => fetchTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: { server: Server.HTTPS_PUBLIC },
  expected: FetchTestResult.SUCCESS,
}), "public to public: no preflight required.");

