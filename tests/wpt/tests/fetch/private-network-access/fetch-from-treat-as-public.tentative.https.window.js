// META: script=/common/utils.js
// META: script=resources/support.sub.js
//
// Spec: https://wicg.github.io/private-network-access/#integration-fetch
//
// These tests verify that documents fetched from the `local` or `private`
// address space yet carrying the `treat-as-public-address` CSP directive are
// treated as if they had been fetched from the `public` address space.

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.OTHER_HTTPS_LOCAL,
    preflight: PreflightBehavior.noPnaHeader(token()),
  },
  expected: FetchTestResult.FAILURE,
}), "treat-as-public-address to local: failed preflight.");

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.OTHER_HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: FetchTestResult.SUCCESS,
}), "treat-as-public-address to local: success.");

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: { server: Server.HTTPS_LOCAL },
  expected: FetchTestResult.SUCCESS,
}), "treat-as-public-address to local (same-origin): no preflight required.");

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: { server: Server.HTTPS_PRIVATE },
  expected: FetchTestResult.FAILURE,
}), "treat-as-public-address to private: failed preflight.");

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.optionalSuccess(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: FetchTestResult.SUCCESS,
}), "treat-as-public-address to private: success.");

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_PUBLIC,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: FetchTestResult.SUCCESS,
}), "treat-as-public-address to public: no preflight required.");
